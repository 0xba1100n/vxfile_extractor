import re
import os
import struct
import zipfile
import sys
import time
import math
import lzma
import shutil
import subprocess
from pathlib import Path
from random import choice, randint
from collections import defaultdict
# 检查当前 Python 版本，3.7 及以上版本使用 text=True，3.6 及以下版本使用 universal_newlines=True
# ---------------------------------------------------------------------------
# 纯 Python 的 strings / grep 替代实现（Windows 原生可用）+ 内容校验
# ---------------------------------------------------------------------------
_NAME_EXTS = re.compile(rb"\.(?:jpg|jpeg|png|gif|js|css|htm|html|cer|pem|xml|bin|txt|ico)$", re.I)


def _iter_strings(data, min_len=5):
    return ((m.start(), m.group()) for m in re.finditer(rb"[ -~]{%d,}" % min_len, data))


def _read(path, limit=64 << 20):
    try:
        with open(path, "rb") as f:
            return f.read(limit)
    except Exception:
        return b""


def _content_ok(path):
    """恢复出来的文件是否真的像它声称的类型（防止把层数据当网页）。"""
    ext = os.path.splitext(path)[1].lower()
    try:
        with open(path, "rb") as f:
            b = f.read(512)
    except Exception:
        return False
    if not b:
        return False
    if ext == ".png":
        return b.startswith(b"\x89PNG")
    if ext == ".gif":
        return b.startswith(b"GIF8")
    if ext in (".jpg", ".jpeg"):
        return b.startswith(b"\xff\xd8")
    if ext in (".htm", ".html"):
        return b.lstrip()[:1] == b"<"
    if ext in (".css", ".js"):
        printable = sum(1 for c in b[:64] if 9 <= c <= 126) / max(1, len(b[:64]))
        if printable <= 0.95:
            return False
        marks = sum(1 for k in (b"{", b";", b"=", b"function", b"var ") if k in b)
        return marks >= 2
    if ext == ".pem":
        return b"-----BEGIN" in b[:200]
    if ext == ".cer":
        return b[:1] == b"\x30"
    if ext == ".xml":
        return b.lstrip().startswith(b"<")
    return False


def _parse_minifs(data):
    """解析 MINIFS 表：[size:4BE][offset:4BE][name:0x50]，stride 0x58。"""
    m = data.find(b"MINIFS")
    if m < 0:
        return []
    start = None
    for p in range(m, min(len(data) - 0x58, m + 0x400)):
        if data[p:p + 1] == b"/" and data[p + 0x40:p + 0x58].count(0) > 12:
            start = p
            break
    if start is None:
        return []
    ents, p = [], start
    while p + 0x58 <= len(data):
        size, off = struct.unpack(">II", data[p - 8:p])
        name = data[p:p + 0x50].split(b"\x00")[0]
        if not name or b"." not in name:
            break
        try:
            ents.append((name.decode("ascii"), size, off))
        except UnicodeDecodeError:
            break
        p += 0x58
        if len(ents) > 4000:
            break
    return ents


def recover_minifs_plaintext(firmware_path, out_base):
    """兜底：minifs 明文段（/conf 等）可直接按 base+offset 拷出。

    表通常在固件本体尾部，而数据在某个解压层里 —— 所以「表」和「数据」要分开找。
    """
    _here = os.path.dirname(os.path.abspath(__file__))
    if os.path.dirname(_here) not in sys.path:
        sys.path.insert(0, os.path.dirname(_here))
    import vxbinwalk as _vb

    raw = _read(firmware_path)
    ents = _parse_minifs(raw)
    if not ents:
        for _o, _d, b in _vb.scan_all(raw):
            ents = _parse_minifs(b)
            if ents:
                break
    if not ents:
        print("[-] MINIFS: no file table found")
        return 0

    span = max(o + sz for n, sz, o in ents)
    text = sorted([(n, o, sz) for n, sz, o in ents
                   if n.lower().endswith((".txt", ".pem", ".xml", ".htm", ".css", ".js"))],
                  key=lambda x: x[1])[:3]
    layers = [b for _o, _d, b in _vb.scan_all(raw) if len(b) > span]
    for blob in layers:
        base = None
        for b0 in range(0, len(blob) - span):
            if all(blob.find(b"\x00", b0 + o, b0 + o + sz) < 0 for n, o, sz in text):
                base = b0
                break
        if base is None:
            continue
        good = 0
        for name, size, off in ents:
            target = os.path.join(out_base, name.lstrip("/"))
            os.makedirs(os.path.dirname(target), exist_ok=True)
            with open(target, "wb") as f:
                f.write(blob[base + off:base + off + size])
            if _content_ok(target):
                good += 1
            else:
                os.remove(target)
        print("[+] MINIFS plaintext segment: base=0x%X, %d files" % (base, good))
        return good
    print("[-] MINIFS: plaintext data base not found")
    return 0

def get_subprocess_params():
    if sys.version_info >= (3, 7):
        return {'text': True}
    else:
        return {'universal_newlines': True}
def get_parent_directory(folder_path):
    """
    获取指定文件夹的父文件夹路径
    """
    parent_directory = os.path.dirname(folder_path)
    return parent_directory
def resolve_binwalk():
    """binwalk 优先用同目录的纯 Python 兼容层 vxbinwalk.py。"""
    env = os.environ.get("VXBINWALK")
    if env and os.path.exists(env):
        return [sys.executable, env]
    here = os.path.dirname(os.path.abspath(__file__))
    for cand in (os.path.join(here, "..", "vxbinwalk.py"), os.path.join(here, "vxbinwalk.py")):
        if os.path.exists(cand):
            return [sys.executable, os.path.abspath(cand)]
    if shutil.which("binwalk"):
        return ["binwalk"]
    return None


BINWALK = resolve_binwalk()


def normalize_input(path):
    """接受 zip/rar/7z：自动取出镜像类成员（而不是体积最大的说明书）。"""
    import tempfile
    try:
        if zipfile.is_zipfile(path):
            with zipfile.ZipFile(path) as z:
                items = [i for i in z.infolist() if not i.is_dir()]
                if items:
                    def rank(i):
                        n = i.filename.lower()
                        return (0 if n.endswith((".bin", ".img", ".trx", ".chk", ".fw", ".rom")) else 1,
                                -i.file_size)
                    it = sorted(items, key=rank)[0]
                    out = path + ".inner.bin"
                    if not os.path.exists(out):
                        with z.open(it) as f, open(out, "wb") as w:
                            shutil.copyfileobj(f, w)
                    print("[*] zip -> %s (%d bytes)"
                          % (it.filename.encode("ascii", "replace").decode(), it.file_size))
                    return out
    except Exception as _e:
        pass
    try:
        head = open(path, "rb").read(8)
    except Exception:
        return path
    seven_cands = [os.environ.get("SEVENZ"),
                   r"C:\\Program Files\\7-Zip\\7z.exe",
                   "/mnt/c/Program Files/7-Zip/7z.exe",
                   shutil.which("7z"), shutil.which("7za")]
    if not (head.startswith(b"Rar!") or head.startswith(b"7z\xbc\xaf")):
        return path
    for seven in seven_cands:
        if not seven or not os.path.exists(str(seven)):
            continue
        tmp = tempfile.mkdtemp()
        try:
            subprocess.run([seven, "x", "-y", "-o" + tmp, path],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=300)
            files = [os.path.join(dp, f) for dp, _, fs in os.walk(tmp) for f in fs
                     if os.path.getsize(os.path.join(dp, f)) > 4096]
            if not files:
                continue
            def rank2(f):
                n = f.lower()
                return (0 if n.endswith((".bin", ".img", ".trx", ".chk", ".fw", ".rom")) else 1,
                        -os.path.getsize(f))
            best = sorted(files, key=rank2)[0]
            print("[*] rar/7z input, extracted with %s: %s (%d bytes)"
                  % (seven, os.path.basename(best), os.path.getsize(best)))
            return best
        except Exception:
            continue
    return path
    return path


def check_binwalk_installed():
    if not BINWALK:
        print("Error: binwalk not found, and no vxbinwalk.py fallback either.")
        sys.exit(1)
    print("Unpack backend: %s" % (" ".join(BINWALK)))

def run_binwalk_extract(file_path):
    """
    执行 binwalk -Me 命令将解压内容存入指定目录，并检查文件是否为未加密镜像。
    同时确认是否为标准的 vxworks5 镜像。
    参数:
    file_path: vxworks固件文件路径
    """
    output_dir = "vxfile_" + os.path.basename(file_path).split('.')[0]
    
    # 判断输出目录是否已经存在，防止重复解包
    if os.path.exists(output_dir):
        print("Output directory exists, skipping unpack: %s" % output_dir)
        extracted_subdir = os.path.join(output_dir, f"_{os.path.basename(file_path)}.extracted")
        if os.path.exists(extracted_subdir):
            output_dir = extracted_subdir
        print("Using existing extraction directory: %s" % output_dir)
    else:
        # 输出目录不存在时，执行解包
        print("Unpacking and checking format/encryption... this may take a minute...")
        command = BINWALK + ['-Me', '-C', output_dir, file_path]
        print("Command: %s" % " ".join(command))
        
        try:
            # 获取 subprocess 参数
            subprocess_params = get_subprocess_params()
            
            result = subprocess.run(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
                **subprocess_params
            )
            
            output = result.stdout
            print("binwalk output:\n%s" % output)
            
            extracted_subdir = os.path.join(output_dir, f"_{os.path.basename(file_path)}.extracted")
            if os.path.exists(extracted_subdir):
                output_dir = extracted_subdir
            
        except subprocess.CalledProcessError as e:
            print("Unpack failed:")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            print("binwalk may be missing or incomplete")
            sys.exit(1)

    # 直接执行 binwalk 命令以获取文件信息
    command = BINWALK + [file_path]
    print("Command: %s" % " ".join(command))

    try:
        # 获取 subprocess 参数
        subprocess_params = get_subprocess_params()
        
        result = subprocess.run(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
            **subprocess_params
        )
        
        output = result.stdout if isinstance(result.stdout, str) else result.stdout.decode('utf-8')
        print("binwalk analysis output:\n%s" % output)
        
        # 检查端序
        endian = "big" if "big endian" in output.lower() else "little" if "little endian" in output.lower() else "unknown"
        print("Detected endianness: %s" % endian)
        if endian == "unknown":
            print("[!] Endianness not detected, assuming big-endian")
            endian = "big" 

        # 输出的三个参数
        return output, output_dir, endian

    except subprocess.CalledProcessError as e:
        print("Analysis failed:")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        print("binwalk may be missing or broken. See:")
        sys.exit(1)


def extract_web_source_filenames(target_directory):
    """从解包出的页面里抽 src="xxx.gif|jpg|js|css" 的文件名。"""
    if not os.path.isdir(target_directory):
        raise ValueError(f"解压目录不存在: {target_directory}")
    names = set()
    pat = re.compile(rb'src\s*=\s*["\']([^"\']{1,200}?\.(?:gif|jpg|js|css))', re.I)
    for dp, _, fs in os.walk(target_directory):
        for f in fs:
            data = _read(os.path.join(dp, f), 8 << 20)
            for m in pat.finditer(data):
                names.add(m.group(1).decode("latin1").split("/")[-1])
    return list(names) if names else False

def calculate_compactness(offsets):
    """
    计算给定偏移量的“紧凑规整程度”。
    直接计算所有偏移量之间的差值的绝对值之和，
    这种逐渐增大的正负“震荡函数”，例如1-2+3-4+5就比3-5+10-22+50显然紧凑，
    而偏移表会显现出更好的“紧凑性”

    :param offsets: 偏移量列表
    :return: 紧凑规整程度
    """
    if len(offsets) < 2:
        return 0
    
    compactness = 0
    # 计算相邻偏移量之间的差值总和
    for i in range(len(offsets) - 1):
        compactness += abs(offsets[i + 1] - offsets[i])/((i+1)/2)
    
    return compactness

def fuzzy_search_file_contain_table(directory_path):
    """按「命中的文件名条数」给解包文件打分，返回 {路径: 分数}。"""
    result = {}
    for f in os.listdir(directory_path):
        fp = os.path.join(directory_path, f)
        if not os.path.isfile(fp):
            continue
        data = _read(fp)
        score = sum(1 for _o, strb in _iter_strings(data) if _NAME_EXTS.search(strb))
        if score:
            result[fp] = score
    if result:
        print("Top name hits: %s" % sorted(result.items(), key=lambda x: -x[1])[:5])
    return result

def get_file_size(file_path):
    """
    获取文件的大小，如果文件不存在返回 -1
    """
    try:
        if not os.path.exists(file_path):  # 检查文件是否存在
            print("File not found: %s" % file_path)
            return -1
        return os.path.getsize(file_path)  # 获取文件大小
    except FileNotFoundError:
        print("Cannot open file: %s" % file_path)
        return -1  # 文件不存在时返回 -1
    except Exception as e:
        print("stat failed: %s (%s)" % (file_path, e))
        return -1

def find_binary_matches(target_directory, filenames):
    """找出同时含有这些文件名的解包文件，返回其中最小的那个（与原实现一致）。"""
    hits = None
    for name in filenames:
        needle = str(name).encode("latin1", "ignore")
        cur = set()
        for dp, _, fs in os.walk(target_directory):
            for f in fs:
                fp = os.path.join(dp, f)
                if needle and needle in _read(fp):
                    cur.add(fp)
        hits = cur if hits is None else (hits & cur)
        if not hits:
            return None
    if not hits:
        return None
    return min(hits, key=os.path.getsize)

def find_files_offset_table(file_path):
    """找「文件偏移表」起点：第一条以 .htm/.jpg/... 结尾的可打印字符串。"""
    data = _read(file_path)
    for off, strb in _iter_strings(data):
        if _NAME_EXTS.search(strb):
            print("[+] File table: %s @ 0x%X" % (file_path, off))
            return off
    print("No matching table found; cannot extract offsets.")
    return None

def extract_file_info_type1(file_path, start_offset, endian='big'):
    """
    type1: 文件名1+"00"*n+文件偏移1+文件名2+"00"*n+文件偏移2 形态
    从指定的偏移量开始提取文件名和偏移信息，返回文件名及其偏移的键值对。
    在找到文件名后，继续向后找非零字符，然后对齐4字节，读取4字节内容作为偏移值。
    如果匹配到的文件名长度达到 0x100，说明已经是接下来的大片程序代码区域而非表格，则放弃继续匹配。
    参数:
    file_path: uImage镜像路径
    start_offset: 偏移表的开头
    endian: 端序，little或big
    """
    print("[*] Scanning entries from 0x%X - 0x50" % start_offset)
    start_offset = start_offset - 0x50 
    file_name_pattern = re.compile(rb'^[A-Za-z0-9_\/\-]*\.[A-Za-z0-9_\/\-]*$')
    file_info = {}
    with open(file_path, 'rb') as f:
        data = f.read()
    start = start_offset
    max_scan_length = 0x10000  
    end_position = min(len(data), start + max_scan_length)

    while start < end_position:
        # 跳过00字节，找到第一个非00字节
        while start < end_position and data[start] == 0x00:
            start += 1
        # 找到第一个非00字节后的4字节对齐位置
        first_non_zero_start = start
        if first_non_zero_start % 4 != 0:
            first_non_zero_start += 4 - (first_non_zero_start % 4)
        # 记录4字节对齐开始的字符串
        temp_str = b''
        while first_non_zero_start < end_position and data[first_non_zero_start] != 0x00:
            temp_str += data[first_non_zero_start:first_non_zero_start + 1]
            first_non_zero_start += 1

        # 检查是否符合文件名模式并且长度是否超过0x100
        if len(temp_str) >= 0x100:
            #print("检测到文件名长度达到0x100，可能已到达表的末尾，停止匹配。")
            start = first_non_zero_start + 1
            continue

        match = file_name_pattern.match(temp_str)
        if match:
            file_name = match.group().decode('ascii', 'ignore')
            #print(f"匹配到的文件名: {file_name}, 位置: {hex(memory_first_non_zero_start)}")

            if len(file_name) >= 5:  # 添加文件名长度的判断,帮助判断文件名合法性
                # 找到文件名后，继续向后跳过00字节
                current_position = first_non_zero_start
                while current_position < end_position and data[current_position] == 0x00:
                    current_position += 1

                # 对齐到下一个4字节边界
                if current_position % 4 != 0:
                    current_position += 4 - (current_position % 4)

                # 读取当前偏移值（4字节）
                if current_position + 4 <= len(data):
                    raw_offset_bytes = data[current_position:current_position + 4]
                    offset_hex = (
                        raw_offset_bytes.hex().lstrip("0").upper() if endian == 'big'
                        else raw_offset_bytes[::-1].hex().lstrip("0").upper()
                    )
                    offset_hex = offset_hex if offset_hex else "0"
                    offset_int = int(offset_hex, 16)
                    adjusted_offset = offset_int #+ filesystem_offset
                    #adjusted_offset = hex(adjusted_offset).lstrip("0x").upper() or "0"
                    # 如果文件名已经存在，则只保留最小的偏移量
                    if file_name in file_info:
                        file_info[file_name] = min(file_info[file_name], adjusted_offset)
                    else:
                        file_info[file_name] = adjusted_offset
                    if os.environ.get("VX_DEBUG"):
                        print("  %-40s 0x%s" % (file_name, hex(file_info[file_name])[2:].upper()))
                start = current_position + 4
            else:
                start = first_non_zero_start + 1
        else:
            start = first_non_zero_start + 1
    if not file_info:
        print("[-] No names/offsets found; format may be unsupported")
    file_info_str = {file_name: str(offset) for file_name, offset in file_info.items()}
    """
    min_offset_file = min(file_info, key=file_info.get) if file_info else None
    if min_offset_file:
        print("[*] Lowest offset entry: %s @ %s" % (min_offset_file, hex(file_info[min_offset_file])))
    """
    return file_info_str


def extract_file_info_type2(file_path, start_offset, endian='big'):
    """
    type2: 文件名1+"00"*1+文件名2+"00"*1+文件名3 
    然后 文件偏移1+"00"*1+文件偏移2+"00"*1+文件偏移3 这种形态
    """
    #print(f"从偏移量 {hex(start_offset)} 处开始提取文件信息, 增加容错率...")
    
    file_info = {}
    
    with open(file_path, 'rb') as f:
        data = f.read()

    # 前向0x100范围内搜索MINIFS字符串
    search_range_start = max(0, start_offset - 0x100)
    search_range_end = start_offset
    minifs_offset = data.find(b'MINIFS', search_range_start, search_range_end)
    
    if minifs_offset != -1:
        start_offset = minifs_offset + 0x20
        print("[+] MINIFS magic found, start offset: 0x%X" % start_offset)
    else:
        print("[*] No MINIFS magic, using original offset: 0x%X" % start_offset)

    start = start_offset
    max_scan_length = 0x10000  # 最大扫描长度
    end_position = min(len(data), start + max_scan_length)

    # 检测ToN_start_offset-N最前面的非空处
    ToN_start_offset = start - 2
    while ToN_start_offset > 0 and data[ToN_start_offset-1] != 0x00:
        ToN_start_offset -= 1

    # 记录当前偏移的4的倍数上取值
    ToN_start_offset = math.ceil(ToN_start_offset / 4) * 4
    print(f"ToN_start_offset: {hex(ToN_start_offset).upper()}")

    # 从ToN_start_offset开始读取数据
    current_position = ToN_start_offset
    ToN_end_offset = None

    while current_position < end_position - 1:  # 需要至少两个字节进行检查
        if data[current_position] != 0x00 and data[current_position + 1] == 0x00 and data[current_position + 2] == 0x00:
            ToN_end_offset = current_position
            break
        current_position += 1

    if ToN_end_offset is not None:
        print("[+] ToN end offset: 0x%X" % ToN_end_offset)
    else:
        print("[-] ToN end not found")

    # 上取4的倍数于ToN_end_offset偏移值
    if ToN_end_offset is not None:
        ToF_start = math.ceil((ToN_end_offset+1) / 4) * 4
        print(f"ToF_start: {hex(ToF_start).upper()}")
    else:
        ToF_start = None

    # 读取ToN_start_offset-12开始的4字节，得到files_count
    if ToN_start_offset >= 12:
        files_count_offset = ToN_start_offset - 12
        files_count = int.from_bytes(data[files_count_offset:files_count_offset + 4], byteorder=endian)
        print(f"files_count: {hex(files_count)}")
    else:
        files_count = None
        print("[-] Cannot read files_count (offset out of range)")

    # 从ToF_start开始读取文件信息
    file_entries = []
    if ToF_start is not None and files_count is not None:
        current_position = ToF_start
        entry_size = 5 * 4  # 每组由5个4字节组成

        for _ in range(files_count):
            if current_position + entry_size <= len(data):
                entry = {
                    'ToN_offset_path': int.from_bytes(data[current_position:current_position + 4], byteorder=endian),
                    'ToN_offset_filename': int.from_bytes(data[current_position + 4:current_position + 8], byteorder=endian),
                    'chunk_number': int.from_bytes(data[current_position + 8:current_position + 12], byteorder=endian),
                    'offset_within_chunk': int.from_bytes(data[current_position + 12:current_position + 16], byteorder=endian),
                    'file_size': int.from_bytes(data[current_position + 16:current_position + 20], byteorder=endian)
                }
                file_entries.append(entry)
                current_position += entry_size
            else:
                print("[-] Not enough data for file entries")
                break

        print("[+] File entries: %d" % len(file_entries))
    else:
        print("[-] Cannot read file entries (ToF_start/files_count invalid)")

    # 从file_entries中读取路径和文件名，构造文件字典
    for entry in file_entries:
        # 从ToN_start_offset + ToN_offset_path读取路径，直到\x00
        path_offset = ToN_start_offset + entry['ToN_offset_path']
        path_end = data.find(b'\x00', path_offset)
        if path_end != -1:
            path = data[path_offset:path_end].decode('utf-8', 'ignore')
        else:
            path = ""

        # 从ToN_start_offset + ToN_offset_filename读取文件名，直到\x00
        filename_offset = ToN_start_offset + entry['ToN_offset_filename']
        filename_end = data.find(b'\x00', filename_offset)
        if filename_end != -1:
            filename = data[filename_offset:filename_end].decode('utf-8', 'ignore')
        else:
            filename = ""

        # 组合路径和文件名作为字典的键
        full_path = f"{path}/{filename}" if path else filename

        # 计算file_offset_in_filesystem
        file_offset_in_filesystem = ToF_start + files_count * 20 + entry['chunk_number'] * 12 + entry['offset_within_chunk']
        if file_offset_in_filesystem + 4 <= len(data):
            value = int.from_bytes(data[file_offset_in_filesystem:file_offset_in_filesystem + 4], byteorder=endian)
            #value_str = str(value)
        else:
            value = None

        # 将键值对存入字典
        if value is not None:
            if full_path in file_info:
                file_info[full_path] = min(file_info[full_path], value)
            else:
                file_info[full_path] = value

    # 打印并返回键值对
    for key, value in file_info.items():
        if os.environ.get("VX_DEBUG"):
            print("  %-40s %s" % (key, hex(value)))

    return file_info



def extract_offsets_from_output(data):
    # 用于存储十进制偏移数组
    decimal_offsets = []
    
    # 正则表达式匹配每行的十进制偏移、十六进制值和描述
    pattern = re.compile(r'(\d+)\s+0x([A-Fa-f0-9]+)\s+(.*)')

    # 标志位，指示是否开始提取数据
    started = False
    lines = data.splitlines()

    # 确定从哪一行开始提取数据
    for idx, line in enumerate(lines):
        if "LZMA compressed data" in line:
            started = True
            start_idx = idx
            break
    else:
        # 如果没有找到 "LZMA compressed data"，从第三行开始
        start_idx = 2
        started = True

    # 遍历从确定行开始的每一行，使用正则表达式匹配
    for line in lines[start_idx:]:
        match = pattern.match(line)
        if match:
            decimal_offset = int(match.group(1))  # 十进制偏移
            decimal_offsets.append(decimal_offset)

    return decimal_offsets



def rename_extracted_files(file_info, extracted_dir, filesystem_offset,binwalk_shell_output):  
    """ 
    根据给定的文件信息复制并重命名解压的文件，创建必要的文件夹结构。
    将文件复制到解压后的根目录之下并保留相对路径结构。

    参数：
    file_info : 文件信息，键为目标文件名，值为调整后的偏移值。
    extracted_dir : 解压后的文件所在的目录。
    filesystem_offset : 文件系统的偏移值。
    binwalk_shell_output: binwalk输出,仅用于提取最后一行的偏移值,用来限制文件名的修复过程
    """
    max_file_offset = 0xFFFFFFFF
    lines = binwalk_shell_output.strip().splitlines()
    if lines:
        last_line = lines[-1].strip()
        match = re.match(r'^(\d+)', last_line)
        if match:
            max_file_offset = int(match.group(1))
    print("[*] Max file offset: 0x%X" % max_file_offset)
    # 输出目录已经是解压后的目录，例如：vxfile_mw313rv4/_mw313rv4.bin.extracted
    print("[*] Trying filesystem offset 0x%X" % filesystem_offset)
    print("[*] Copying files into result directory...")
    # 正确数大于10则再也无视错误
    true_count = 0
    filesystem_offset_is_true = 0
    #一直错则有可能不是这个文件偏移
    false_count = 0    
    for target_name, adjusted_offset in file_info.items():
        try:
            # 将十六进制偏移值转换为整数并加上文件系统偏移量
            original_offset = int(adjusted_offset) + filesystem_offset
            original_offset_hex = hex(original_offset).lstrip("0x").upper()  # 转换为八位十六进制字符串，去掉 0x 前缀，补足前导零
            #print(f"文件: {target_name}, 相对文件系统偏移值: {hex(adjusted_offset)}, 加文件系统偏移后的偏移值: {original_offset_hex}")  #DEBUG用
        except ValueError as e:
            print("[-] Invalid offset %s, skipped (%s)" % (adjusted_offset, e))
            continue
        if(original_offset>max_file_offset):
            continue
        # 生成旧的文件路径（以偏移值为文件名，在解压后的目录中）
        old_file_path = os.path.join(extracted_dir, original_offset_hex)
        if not re.fullmatch(r"[\x20-\x7e]{1,200}", str(target_name)) or ".." in str(target_name):
            false_count += 1
            continue
        # 生成新的文件路径 (解压路径/result_vxworks_file/binwalk把内存偏移所在作为文件的名称)
        new_file_path = os.path.join(get_parent_directory(extracted_dir) + "/","result_vxworks_file",target_name.lstrip(os.sep))
        # 如果目标文件名包含路径，创建相应的目录结构
        target_directory = os.path.dirname(new_file_path)
        if target_directory and not os.path.exists(target_directory):
            os.makedirs(target_directory, exist_ok=True)

        # 复制文件
        if os.path.exists(old_file_path):
            try:
                shutil.copy2(old_file_path, new_file_path)
                if not _content_ok(new_file_path):
                    try:
                        os.remove(new_file_path)
                    except OSError:
                        pass
                    false_count += 1
                    continue
                if os.environ.get("VX_DEBUG"):
                    print("  renamed %s -> %s" % (old_file_path, new_file_path))
                true_count += 1
            except IOError as e:
                print("[-] Copy failed: %s -> %s (%s)" % (old_file_path, new_file_path, e))
                pass
        else:
            false_count += 1
            print("[-] Missing file: %s" % old_file_path)

        if(true_count>=5):
            filesystem_offset_is_true = 1
        
        if((false_count>=10) & (filesystem_offset_is_true == 0)):
            print("[!] Filesystem offset 0x%X looks wrong, trying another" % filesystem_offset)
            return filesystem_offset_is_true
        
    return filesystem_offset_is_true
    
def check_if_firmware_itself_have_table(firmware_path):
    """固件本体里是否直接带着文件偏移表（>=10 条文件名）。"""
    data = _read(firmware_path)
    n = sum(1 for _off, strb in _iter_strings(data) if _NAME_EXTS.search(strb))
    return n > 10

def check_crypted_fileoffset_table(folder_path):
    """偏移表被加密/隐藏的固件特征串：只警告，不再直接退出。"""
    for dp, _, fs in os.walk(folder_path):
        for f in fs:
            data = _read(os.path.join(dp, f))
            if b"Decryption for config.bin" in data or b"des_min_do" in data:
                print("[!] Found 'Decryption for config.bin'; offset table may be encrypted, retrying")
                return

def decide_extract_mode(file_path, infile_offset, endian):
    """
    决定哪种偏移表提取方式
    mode 1: 文件名1+"00"*n+文件偏移1+文件名2+"00"*n+文件偏移2 形态
    mode 2: 文件名1+"00"*1+文件名2+"00"*1+文件名3  
            文件偏移1+"00"*1+文件偏移2+"00"*1+文件偏移3  
            形态
    """
    mode = 2  # 默认设置为 mode 2
    
    try:
        with open(file_path, 'rb') as file:
            # 移动到指定偏移位置
            file.seek(infile_offset)
            
            # 检索0x50字节的数据
            bytes_to_read = 0x50
            data = file.read(bytes_to_read)
            
            # 查找连续的 0x00 00 00 00 字节
            if b'\x00\x00\x00\x00' in data:
                mode = 1
    
    except FileNotFoundError:
        print(f"Error: File '{file_path}' not found.")
    except Exception as e:
        print(f"Error: {e}")
    
    return mode


def extract_function_table(firmware_path, output_path):
    """在固件本体与所有解包层里找 VxWorks 符号表（5A/5D/6E 三种 LZMA props）。"""
    if not firmware_path:
        print("Usage: provide a firmware file path")
        return None

    content = _read(firmware_path)

    # 优先用独立模块 vxsym（支持未压缩符号表、NUL 对齐、裸 LZMA1 等）
    _here = os.path.dirname(os.path.abspath(__file__))
    if os.path.dirname(_here) not in sys.path:
        sys.path.insert(0, os.path.dirname(_here))
    try:
        import vxsym
        hit = vxsym.recover_symbol_table(content)
        if hit:
            _base = re.sub(r'_[^/\\]+\.extracted$', '', output_path)
            _name = "SYMBOL_Table"
            _path = os.path.join(_base, _name)
            _d = os.path.dirname(_path)
            if _d:
                os.makedirs(_d, exist_ok=True)
            with open(_path, "wb") as _f:
                _f.write(hit["blob"])
            _names = sorted({str(n) for n in (hit.get("names") or [])})
            # vxsym 的 names 字段只带示例名；需要完整清单时从表体里重新抽
            if len(_names) < int(hit.get("names_count") or 0):
                _names = sorted({m.decode("latin1") for m in
                                 re.findall(rb"[A-Za-z_][A-Za-z0-9_]{3,}", hit["blob"])})
            if _names:
                with open(os.path.join(_base, "SYMBOL_Names.txt"), "w", encoding="utf-8") as _nf:
                    _nf.write("\n".join(_names) + "\n")
            print("[+] Symbol name table (vxsym/%s @0x%X): %d names, %d bytes -> %s"
                  % (hit.get("detector", "?"), hit.get("offset", 0),
                     hit.get("names_count", len(_names)), len(hit["blob"]), _path))
            return _path
    except ImportError:
        pass
    except Exception as _e:
        print("[-] vxsym failed, using builtin fallback: %s" % _e)

    blobs = [("firmware", content)]
    _here = os.path.dirname(os.path.abspath(__file__))
    if os.path.dirname(_here) not in sys.path:
        sys.path.insert(0, os.path.dirname(_here))
    try:
        import vxbinwalk as _vb
        for off, _desc, payload in _vb.scan_all(content):
            if payload and len(payload) > 4096:
                blobs.append(("layer@0x%X" % off, payload))
    except Exception as e:
        print("[-] Container decoder unavailable, scanning raw only: %s" % e)

    MARKERS = (b"bzero", b"usrRoot", b"taskSpawn", b"windFindNextSym", b"WIND version")
    best = None
    seen = set()
    for name, blob in blobs:
        if len(blob) < 4096:
            continue
        for m in range(0, len(blob) - 16):
            if blob[m] not in (0x5A, 0x5D, 0x6E) or blob[m + 1:m + 3] != b"\x00\x00":
                continue
            try:
                dec = lzma.decompress(blob[m:], format=lzma.FORMAT_ALONE)
            except Exception:
                continue
            if len(dec) < 4096 or dec[:4] in seen:
                continue
            seen.add(dec[:4])
            score = sum(1 for k in MARKERS if k in dec)
            if score >= 2 or b"bzero" in dec:
                if best is None or score > best[0]:
                    best = (score, name, dec, m)

    if best is None:
        print("[-] Symbol name table: not found")
        return None

    score, name, dec, off = best
    base_output_path = re.sub(r'_[^/\\]+\.extracted$', '', output_path)
    symbol_table_path = os.path.join(base_output_path, "SYMBOL_Table")
    d = os.path.dirname(symbol_table_path)
    if d:
        os.makedirs(d, exist_ok=True)
    with open(symbol_table_path, "wb") as f:
        f.write(dec)
    # 顺手导出纯符号名清单（IDA/Python 里直接可用）
    try:
        names = sorted({m.decode("latin1") for m in re.findall(rb"[A-Za-z_][A-Za-z0-9_]{4,}", dec)})
        names = [n for n in names if len(n) <= 64]
        with open(os.path.join(os.path.dirname(symbol_table_path), "SYMBOL_Names.txt"), "w",
                  encoding="utf-8") as nf:
            nf.write("\n".join(names) + "\n")
        print("[+] Symbol names: %d -> SYMBOL_Names.txt" % len(names))
    except Exception as _e:
        print("[-] Symbol name export failed: %s" % _e)
    print("[+] Symbol table from %s @0x%X (%d hits, %d bytes) -> %s"
          % (name, off, score, len(dec), symbol_table_path))
    return symbol_table_path

def find_max_uncompressed_offset(binwalk_output):
    # 正则表达式匹配每一行包含偏移和uncompressed size的信息,最大的那个解压后大小的文件就是主程序，返回其偏移量
    pattern = re.compile(r"(\d+)\s+(0x[0-9A-Fa-f]+)\s+.*uncompressed size:\s+(-?\d+) bytes")
    
    max_uncompressed_size = -1
    max_offset = None

    # 逐行查找匹配
    for match in pattern.finditer(binwalk_output):
        decimal_offset = int(match.group(1))
        hex_offset = match.group(2)
        uncompressed_size = int(match.group(3))

        # 检查uncompressed size是否为正值并且比当前最大值大
        if uncompressed_size > max_uncompressed_size:
            max_uncompressed_size = uncompressed_size
            max_offset = hex_offset

    return max_offset

def print_output_hint(vxfile_directory):
    """bin/ src/ cfg/ 分类 + 主逻辑提示。"""
    try:
        import classify as _cls
        _root = get_parent_directory(vxfile_directory)
        # 放到被统计的树外层，避免 run_extractor 把硬链接再数一遍
        _b = os.path.basename(_root)
        if _b == "vxfile_input" or _b.startswith("vxfile_"):
            _root = os.path.dirname(_root) or "."
        print(_cls.format_hint(_cls.classify(_root)))
    except Exception as e:
        print("[-] classify failed: %s" % e)


def payload_extract(data, out_dir):
    """原始数据 + 各解压层，依次试 MINIFS / OW / PK11 / 裸内核。"""
    try:
        import fs_carve
        res = os.path.join(out_dir, "result_vxworks_file")
        hit = fs_carve.extract_all(data, res)
        if not hit:
            import vxbinwalk as vb
            for off, _kind, blob in vb.scan_all(data):
                if len(blob) < 65536:
                    continue
                hit = fs_carve.extract_all(blob, res)
                if hit:
                    print("[+] Payload unpacked (%s, layer 0x%X): %d file(s)"
                          % (hit["label"], off, hit["count"]))
                    break
        else:
            print("[+] Payload unpacked (%s): %d file(s)" % (hit["label"], hit["count"]))
        return hit
    except Exception as e:
        print("[-] Payload unpack failed: %s" % e)
        return None


def dump_minifs_table(firmware_path, out_dir):
    """导出文件名表（多布局）：MINIFS 定长/名字池、stride 型、Linksys TROC 等。

    先用独立模块 vxtable（支持 6 种布局），失败再回退到内置 MINIFS 解析。
    """
    _here = os.path.dirname(os.path.abspath(__file__))
    if os.path.dirname(_here) not in sys.path:
        sys.path.insert(0, os.path.dirname(_here))
    data = _read(firmware_path)

    # 无论 vxtable 有没有结果，都试一遍 MINIFS 解包（v1 / 分块 / v2 / 兜底）
    try:
        import minifs_lzma
        _res = os.path.join(out_dir, "result_vxworks_file")
        _files = minifs_lzma.extract(data, _res)
        _ok = sum(1 for _n, _s, _o in _files if _o)
        if _ok:
            print("[+] MINIFS per-file LZMA: %d / %d extracted" % (_ok, len(_files)))
        else:
            _files = minifs_lzma.extract_chunked(data, _res)
            _ok = sum(1 for _n, _s, _o in _files if _o)
            if _ok:
                print("[+] MINIFS chunked LZMA: %d / %d extracted" % (_ok, len(_files)))
            else:
                _v2 = minifs_lzma.extract_v2(data, _res)
                if _v2:
                    print("[+] MINIFS v2: %d files extracted" % len(_v2))
                else:
                    import lzma_carve
                    _streams = lzma_carve.carve(data, _res)
                    if _streams:
                        print("[+] LZMA stream fallback: %d streams" % len(_streams))
    except ImportError:
        pass
    except Exception as _e:
        print("[-] MINIFS unpack failed: %s" % _e)


    try:
        import vxtable
        # 原始字节 + 解压层都要找：不少型号的表藏在层里
        cands = [(0, data)]
        try:
            import vxbinwalk as _vb
            for _o, _d, _b in _vb.scan_all(data):
                if len(_b) > 65536:
                    cands.append((_o, _b))
        except Exception:
            pass
        tables = []
        for _base, _blob in cands[:12]:
            for _t in vxtable.find_tables(_blob):
                tables.append((_base, _t))
        if tables:
            _b, t = max(tables, key=lambda z: (len(z[1].entries),
                                               getattr(z[1], "confidence", 0)))
            os.makedirs(out_dir, exist_ok=True)
            path = os.path.join(out_dir, "MINIFS_Table.csv")
            with open(path, "w", encoding="utf-8", newline="") as f:
                f.write("name,size,offset\n")
                for name, size, offset in t.entries:
                    _off = (offset + _b) if isinstance(offset, int) and offset >= 0 else offset
                    f.write("%s,%s,%s\n" % (str(name).replace(",", "_"), size, _off))
            print("[+] Filename table (vxtable/%s): %d entries -> %s"
                  % (getattr(t, "layout", "?"), len(t.entries), path))
            return len(t.entries)
    except ImportError:
        pass
    except Exception as _e:
        print("[-] vxtable failed, using builtin MINIFS parser: %s" % _e)

    for blob in [data]:
        ents = _parse_minifs(blob)
        if not ents:
            continue
        os.makedirs(out_dir, exist_ok=True)
        path = os.path.join(out_dir, "MINIFS_Table.csv")
        with open(path, "w", encoding="utf-8", newline="") as f:
            f.write("name,size,offset\n")
            for name, size, off in ents:
                f.write("%s,%d,%d\n" % (name.replace(",", "_"), size, off))
        print("[+] MINIFS file table: %d entries -> %s" % (len(ents), path))
        return len(ents)

    # 兜底：没有结构化表，就从镜像与解压层里收割真实文件名（只给名字，不给偏移）
    try:
        import vxbinwalk as _vb
        blobs = [data] + [b for _o, _d, b in _vb.scan_all(data) if len(b) > 8192]
        name_re = re.compile(rb"[A-Za-z0-9_\-./]{3,80}\.(?:htm|html|cgi|js|css|gif|jpg|jpeg|png|xml|asp|ico)")
        found = set()
        for blob in blobs:
            for m in name_re.finditer(blob):
                s_name = m.group().decode("latin1")
                if s_name.count("/") > 6 or s_name.startswith("/"):
                    continue
                found.add(s_name)
        if len(found) >= 20:
            os.makedirs(out_dir, exist_ok=True)
            path = os.path.join(out_dir, "NAMES_ONLY.csv")
            with open(path, "w", encoding="utf-8", newline="") as f:
                f.write("name,size,offset\n")
                for n in sorted(found):
                    f.write("%s,,\n" % n.replace(",", "_"))
            print("[+] Name list only (no offsets): %d entries -> %s" % (len(found), path))
            return len(found)
    except Exception as _e:
        print("[-] Name harvest failed: %s" % _e)
    return 0


def write_summary(out_dir, firmware_path, binwalk_output, symbol_path, recovered, valid):
    """写 SUMMARY.json：层数、VxWorks 版本、符号表、恢复情况。"""
    import json
    layers = []
    for line in binwalk_output.splitlines():
        m = re.match(r"^\s*(\d+)\s+0x([0-9A-Fa-f]+)\s+(.*)", line)
        if m:
            layers.append({"offset": int(m.group(1)), "desc": m.group(3).strip()[:120]})
    raw = _read(firmware_path, 16 << 20)
    vers = {v.decode() for v in re.findall(rb"VxWorks\s?([0-9]+\.[0-9]+(?:\.[0-9]+)?)", raw)}
    if symbol_path and os.path.exists(symbol_path):
        vers |= {v.decode() for v in
                 re.findall(rb"VxWorks\s?([0-9]+\.[0-9]+(?:\.[0-9]+)?)", _read(symbol_path, 8 << 20))}
    info = {
        "input": os.path.basename(firmware_path),
        "size": os.path.getsize(firmware_path) if os.path.exists(firmware_path) else 0,
        "layers": len(layers),
        "layer_detail": layers[:40],
        "vxworks_versions": sorted(vers),
        "symbol_table": os.path.basename(symbol_path) if symbol_path else None,
        "recovered_named": recovered,
        "recovered_valid": valid,
    }
    try:
        os.makedirs(out_dir, exist_ok=True)
        with open(os.path.join(out_dir, "SUMMARY.json"), "w", encoding="utf-8") as f:
            json.dump(info, f, ensure_ascii=False, indent=1)
        print("[+] SUMMARY.json: layers=%d, vx=%s, symbol=%s, named=%d, valid=%d"
              % (len(layers), ",".join(info["vxworks_versions"]) or "-",
                 info["symbol_table"] or "-", recovered, valid))
    except Exception as e:
        print("[-] SUMMARY.json write failed: %s" % e)
    return info


def main(file_path,fuzzymode):
    # Windows 控制台是 GBK，文件名里的非 GBK 字符会让 print 抛异常，先兜住
    try:
        sys.stdout.reconfigure(errors="replace")
        sys.stderr.reconfigure(errors="replace")
    except Exception:
        pass
    # 检查 binwalk 是否安装
    check_binwalk_installed()
    file_path = normalize_input(file_path)
    try:
        # 运行 binwalk 并获取解压目录
        binwalk_shell_output, vxfile_directory, endian = run_binwalk_extract(file_path)
        function_offset_table = extract_function_table(file_path,vxfile_directory)
        main_program_offset = find_max_uncompressed_offset(binwalk_shell_output)
        if not main_program_offset:
            cands = []
            for dp, _, fs in os.walk(vxfile_directory):
                for f in fs:
                    fp = os.path.join(dp, f)
                    cands.append((os.path.getsize(fp), f))
            main_program_offset = ("0x" + max(cands)[1]) if cands else "0x0"
        main_program_name = str(main_program_offset).lstrip("0x").upper()
        print("[*] Main program: %s/%s" % (vxfile_directory, main_program_name))

        # 先做一次独立的载荷解包：没有偏移表的样本（裸内核、OW、PK11）也走这里
        try:
            with open(file_path, "rb") as _f:
                payload_extract(_f.read(), get_parent_directory(vxfile_directory))
        except Exception as _e:
            print("[-] Payload unpack failed: %s" % _e)
        check_crypted_fileoffset_table(vxfile_directory)
        
        # 有些固件直接就在本身就有文件偏移表了,会省不少功夫，如C80v1
        firm_itself_have_the_table = check_if_firmware_itself_have_table(file_path)
        if firm_itself_have_the_table:
            best_matching_file = file_path
        # 大多数固件的文件偏移表还是在解包的内容里面的
        else:
            # 尝试提取目标目录中的web资源文件名以寻找偏移表
            contained_filenames = extract_web_source_filenames(vxfile_directory)        
            if contained_filenames and not fuzzymode:
            # 提取web资源文件名成功，那就使用精确的方案
                best_matching_file = find_binary_matches(vxfile_directory, contained_filenames)
            else:
                # 提取web资源文件名失败，那就转而使用次精确的字符串匹配方案
                print("[*] No web asset names found (or fuzzy mode); falling back to string matching")
                best_matching_file = fuzzy_search_file_contain_table(vxfile_directory)
                if isinstance(best_matching_file, dict):
                    best_matching_file = (max(best_matching_file, key=best_matching_file.get)
                                          if best_matching_file else None)
                if not best_matching_file:
                    print("[-] Offset table not found in extraction output; stopping here.")
                    print_output_hint(vxfile_directory)
                    return
        
        infile_offset = find_files_offset_table(best_matching_file)
        file_info = {}
        if infile_offset is None:
            print("[-] No file offset table: skipping rename (list and summary still exported)")
        else:
            mode = decide_extract_mode(file_path, infile_offset, endian)
            if mode == 1:
                file_info = extract_file_info_type1(file_path, infile_offset, endian)
            if mode == 2:
                file_info = extract_file_info_type2(best_matching_file, infile_offset, endian)
            maybe_filesystem_offsets = extract_offsets_from_output(binwalk_shell_output)
            if maybe_filesystem_offsets:
                for testing_filesys_offset in maybe_filesystem_offsets:
                    print("[*] Trying filesystem offset 0x%X" % testing_filesys_offset)
                    if rename_extracted_files(file_info, vxfile_directory, testing_filesys_offset,
                                              binwalk_shell_output) == True:
                        break
        try:
            dump_minifs_table(file_path, get_parent_directory(vxfile_directory))
        except Exception as _e:
            print("[-] MINIFS table export failed: %s" % _e)

        # minifs 明文段兜底：实测只有 /conf/oem.txt 这类条目能对上，
        # 其余条目另有偏移原点，强拷会产出错位内容 —— 默认关闭，需要时用 --minifs 打开。
        if "--minifs" in sys.argv and not os.path.isdir(
                os.path.join(get_parent_directory(vxfile_directory), "result_vxworks_file")):
            try:
                recover_minifs_plaintext(file_path, os.path.join(get_parent_directory(vxfile_directory),
                                                                 "result_vxworks_file"))
            except Exception as _e:
                print("[-] MINIFS fallback recovery failed: %s" % _e)

        _res_dir = os.path.join(get_parent_directory(vxfile_directory), "result_vxworks_file")
        _named = sum(len(fs) for _dp, _dn, fs in os.walk(_res_dir)) if os.path.isdir(_res_dir) else 0
        _valid = 0
        if os.path.isdir(_res_dir):
            for _dp, _dn, fs in os.walk(_res_dir):
                for _f in fs:
                    if _content_ok(os.path.join(_dp, _f)):
                        _valid += 1
        write_summary(get_parent_directory(vxfile_directory), file_path, binwalk_shell_output,
                      function_offset_table, _named, _valid)

        if function_offset_table:
            print("[+] Symbol name table: %s" % function_offset_table)
        else:
            print("[-] Symbol name table: not found (names only, no addresses)")

        print_output_hint(vxfile_directory)

        print("[*] Extracted tree: %s" % vxfile_directory)
    except (ValueError, RuntimeError) as e:
        print("Error: %s" % e)


if __name__ == "__main__":
    # 检查是否请求了帮助信息
    if "-h" in sys.argv or "--help" in sys.argv or len(sys.argv) < 2:
        help_message = """
Usage:
    python3 vxfile_extracter.py <firmware> [--fuzzmode]

Options:
    -h, --help      show this help
    --fuzzmode      force fuzzy matching

Notes:
    Unpacks VxWorks firmware and restores original filenames.
    Exact matching is used by default; --fuzzmode forces fuzzy matching.
        """
        print(help_message)
        sys.exit(0)

    # 输出 ASCII 艺术字
    ascii_art = """
    ██╗   ██╗██╗  ██╗███████╗██╗██╗     ███████╗        ███████╗██╗  ██╗████████╗██████╗  █████╗  ██████╗████████╗ ██████╗ ██████╗ 
    ██║   ██║╚██╗██╔╝██╔════╝██║██║     ██╔════╝        ██╔════╝╚██╗██╔╝╚══██╔══╝██╔══██╗██╔══██╗██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗
    ██║   ██║ ╚███╔╝ █████╗  ██║██║     █████╗          █████╗   ╚███╔╝    ██║   ██████╔╝███████║██║        ██║   ██║   ██║██████╔╝
    ╚██╗ ██╔╝ ██╔██╗ ██╔══╝  ██║██║     ██╔══╝          ██╔══╝   ██╔██╗    ██║   ██╔══██╗██╔══██║██║        ██║   ██║   ██║██╔══██╗
     ╚████╔╝ ██╔╝ ██╗██║     ██║███████╗███████╗███████╗███████╗██╔╝ ██╗   ██║   ██║  ██║██║  ██║╚██████╗   ██║   ╚██████╔╝██║  ██║
      ╚═══╝  ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
    """
    print("\033[1;32m" + ascii_art + "\033[0m")
    time.sleep(1.14514)

    # 解析 fuzzmode 参数
    fuzzymode = "--fuzzymode" in sys.argv

    # 调用主函数
    main(sys.argv[1], fuzzymode)
