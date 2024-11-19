import re
import subprocess
import sys
import os
import shutil
import time
from collections import defaultdict
# 检查当前 Python 版本，3.7 及以上版本使用 text=True，3.6 及以下版本使用 universal_newlines=True
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
def check_binwalk_installed():
    if not shutil.which("binwalk"):
        print("错误：未找到 binwalk，请确保 binwalk 已正确安装。")
        sys.exit(1)
    else:
        print("binwalk 已正确安装。")

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
        print(f"输出目录 {output_dir} 已存在，跳过解包。")
        extracted_subdir = os.path.join(output_dir, f"_{os.path.basename(file_path)}.extracted")
        if os.path.exists(extracted_subdir):
            output_dir = extracted_subdir
        print(f"使用已有解压目录：{output_dir}")

        # 直接执行 binwalk 命令以获取文件信息
        command = ['binwalk', file_path]
        # print(f"执行命令: {' '.join(command)}")

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
            print(f"binwalk 分析输出:\n{output}")
            
            # 检查端序
            endian = "big" if "big endian" in output.lower() else "little" if "little endian" in output.lower() else "unknown"
            print(f"检测到的端序：{endian}")

            # 输出的三个参数
            return output, output_dir, endian

        except subprocess.CalledProcessError as e:
            print(f"分析失败了，错误信息如下：")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            print("binwalk 可能没有正确安装或者发生了其他错误。请访问下列网址获取安装教程：")
            sys.exit(1)

    # 输出目录不存在时，执行解包
    print("开始解包文件，并检查文件格式和加密状态...")
    command = ['binwalk', '-Me', '-C', output_dir, file_path]
    print(f"执行命令: {' '.join(command)}")
    
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
        print(f"binwalk 解包输出:\n{output}")
        is_unencrypted_image = "uImage" in output or "U-Boot" in output
        is_standard_vxworks = "VxWorks" in output or "Wind River" in output
        
        if is_unencrypted_image:
            print("文件中包含 'uImage' 或 'U-Boot'，判断为未加密的镜像文件。")
        else:
            print("未检测到 'uImage' 或 'U-Boot'，文件可能被加密或采用了其他格式。")
        
        if not is_standard_vxworks:
            print("注意：这可能不是标准的 vxworks5 镜像。")
        
        extracted_subdir = os.path.join(output_dir, f"_{os.path.basename(file_path)}.extracted")
        if os.path.exists(extracted_subdir):
            output_dir = extracted_subdir
        
        endian = "big" if "big endian" in output.lower() else "little" if "little endian" in output.lower() else "unknown"
        print(f"检测到的大小端：{endian}")
        
        # 输出的三个参数
        return output, output_dir, endian
    except subprocess.CalledProcessError as e:
        print(f"解包失败了，错误信息如下：")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        print("binwalk 可能没有正确完整安装")
        sys.exit(1)

def extract_web_source_filenames(target_directory):
    """
    对于有页面的固件,提取那些100%正确的文件名以判断文件偏移表所在
    参数: 
    target_directory: 解压目录
    """
    if not os.path.isdir(target_directory):
        raise ValueError(f"解压目录不存在: {target_directory}")
    
    # 执行 shell 指令并获取输出，指定目标目录
    command = f"grep -r \"src=\" {target_directory} | grep -E \"\.(gif|jpg|js|css)\""
    try:
        subprocess_params = get_subprocess_params()
        result = subprocess.run(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True, **subprocess_params)
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"命令执行失败: {e}")
    
    shell_output = result.stdout

    # 如果命令结果为空或只包含换行符等，返回 False
    if not shell_output.strip():
        return False

    # 匹配文件名的正则表达式，匹配 gif, jpg, js, css 结尾的文件
    pattern = r'\b([a-zA-Z0-9_\/]+\.(?:gif|jpg|js|css))\b'
    matches = re.findall(pattern, shell_output)
    
    # 提取文件名并去重
    filenames = set()
    for match in matches:
        # 只获取文件名部分，去掉路径
        filename = match.split('/')[-1]
        filenames.add(filename)
    
    # 转换为数组并返回
    return list(filenames)

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
    """
    遍历目录中的文件，基于 MIME 类型筛选，排除特定类型文件，筛选后文件运行 strings 命令，
    按行数统计匹配结果并计算紧凑程度，根据 "匹配行数*10 / 紧凑程度" 比值进行排序。
    
    :param directory_path: 目标目录路径
    :return: 排序后的字典，键为路径，值为匹配行数与紧凑+整齐程度比值
    """
    result = {}
    
    # 排除的 MIME 类型
    excluded_mime_types = {
        "inode/directory",
        "text/html",
        "application/xml",
        "application/json",
        "text/css",
    }

    try:
        # 获取目录中的文件列表
        files = os.listdir(directory_path)

        for file in files:
            file_path = os.path.join(directory_path, file)

            if os.path.isfile(file_path):
                try:
                    # 使用 file 命令获取 MIME 类型
                    mime_command = f"file --mime-type \"{file_path}\""
                    mime_process = subprocess.run(
                        mime_command,
                        shell=True,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                        universal_newlines=True
                    )
                    if mime_process.returncode == 0:
                        mime_type = mime_process.stdout.split(":")[1].strip()
                        if mime_type in excluded_mime_types:
                            #print(f"跳过文件（MIME 类型排除）：{file_path} [{mime_type}]")
                            continue
                    else:
                        #print(f"无法获取 MIME 类型，跳过文件：{file_path}")
                        continue

                    # 执行 strings 命令并计算匹配行数
                    strings_command = f"strings -t x -n 5 {file_path} | grep -E '\\.jpg|\\.png|\\.js|\\.cer|\\.pem|\\.bin'"
                    process = subprocess.run(
                        strings_command,
                        shell=True,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                        universal_newlines=True
                    )

                    matching_lines = process.stdout.strip().split("\n")
                    match_count = len(matching_lines) if matching_lines[0] else 0

                    # 获取偏移量并计算紧凑程度
                    offsets = [int(re.match(r"^\s*([0-9a-fA-F]+)", line).group(1), 16) for line in matching_lines if re.match(r"^\s*([0-9a-fA-F]+)", line)]
                    compactness = calculate_compactness(offsets)

                    if compactness > 0:
                        # 计算匹配次数与紧凑程度的比值
                        # 使用*1000加权来抹平量级差距，下同
                        ratio = match_count*0x1000 / compactness
                    else:
                        ratio = match_count*0x1000  # 如果紧凑+整齐程度为0，直接使用匹配次数作为比值

                    # 记录结果（包括路径、匹配次数、紧凑+整齐程度、比值）
                    result[file_path] = (match_count, compactness, ratio)

                except Exception as e:
                    print(f"处理文件 {file_path} 时出错：{e}")

            elif os.path.isdir(file_path):  # 跳过文件夹
                #print(f"跳过文件夹：{file_path}")
                continue
            else:
                #print(f"跳过无效文件：{file_path}")
                continue

        # 计算最大匹配行数
        max_match_count = max([result[file_path][0] for file_path in result]) if result else 0

        # 只选择匹配行数大于5 的文件
        result = {file_path: values for file_path, values in result.items() if values[0] >= 5}

        # 按照比值排序（从高到低），并在比值相同的情况下按文件名长度升序排列
        sorted_result = dict(sorted(result.items(), key=lambda item: (item[1][2], -len(item[0])), reverse=True))

        # 输出排序后的前几个文件
        print("根据匹配行数 / 紧凑程度的比值排序后的结果：")
        for file_path, (count, compactness, ratio) in sorted_result.items():
            print(f"{file_path} - 匹配行数: {count}, 紧凑+整齐程度: {compactness}, 比值: {ratio:.4f}")

        # 获取并返回排序后的第一个文件
        best_match_file = list(sorted_result.items())[0]  # 获取排序后的第一个文件
        print(f"最优匹配文件: {best_match_file[0]}")
        return best_match_file[0]

    except Exception as e:
        print(f"处理目录时出错：{e}")
        return {}


def get_file_size(file_path):
    """
    获取文件的大小，如果文件不存在返回 -1
    """
    try:
        if not os.path.exists(file_path):  # 检查文件是否存在
            print(f"文件不存在: {file_path}")
            return -1
        return os.path.getsize(file_path)  # 获取文件大小
    except FileNotFoundError:
        print(f"无法找到文件: {file_path}")
        return -1  # 文件不存在时返回 -1
    except Exception as e:
        print(f"获取文件大小时出错: {file_path}, 错误: {e}")
        return -1

def find_binary_matches(target_directory, filenames):
    """
    包含http服务固件的特有方案，其使用web静态资源引用的文件名来对包含文件偏移表的可能文件进行查找
    """
    matching_files = None
    print(f"开始查找目标目录 {target_directory} 中的二进制文件...")

    for filename in filenames:
        #print(f"\n正在查找文件名: {filename}")
        
        # 执行 grep 查找包含文件名的二进制文件
        command = f"grep -r {filename} {target_directory} | grep Binary"
        #print(f"执行命令: {command}")
        
        try:
            subprocess_params = get_subprocess_params()
            result = subprocess.run(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True, **subprocess_params)
            #print("命令执行成功，开始处理输出...")
        except subprocess.CalledProcessError:
            #print(f"没有找到匹配的文件，跳过文件: {filename}")
            continue
        
        # 检查 stdout 是否为字节类型，若是则解码为字符串
        if isinstance(result.stdout, bytes):
            #print("输出为字节类型，正在解码...")
            shell_output = result.stdout.decode('utf-8')  # 需要解码
        else:
            #print("输出已经是字符串类型，无需解码。")
            shell_output = result.stdout  # 已经是字符串类型，无需解码

        # 输出命令结果
        #print(f"命令输出:\n{shell_output}")
        
        # 提取匹配到的文件
        pattern = r'Binary file ([^\s]+) matches'
        matches = re.findall(pattern, shell_output)
        #print(f"匹配到的文件: {matches}")
        
        # 如果没有匹配到任何文件，继续下一次循环
        if not matches:
            print(f"没有匹配到任何二进制文件，继续查找下一个文件名。")
            continue

        # 将匹配到的文件转换为集合
        current_matches = set(matches)
        #print(f"当前匹配文件：{current_matches}")
        
        if matching_files is None:
            matching_files = current_matches
        else:
            # 求交集
            matching_files.intersection_update(current_matches)
    print(f"\033[32m最后匹配到的存在文件偏移表文件集：{matching_files}\033[0m")

    # 如果没有匹配到任何文件，返回空列表
    if matching_files is None:
        print("找不到binwalk -Me解压后的、明文可接触的文件偏移表，很可能该表已被加密或进一步压缩。")
        print("尝试手动解密分析固件，然后把该表的二进制形式放在解压文件夹里，仍旧可以正常恢复文件名。")
        return []

    # 比较文件大小并返回最小的文件
    smallest_size = float('inf')  # 初始化为无穷大
    smallest_file = None

    for match in matching_files:
        # 对每个匹配的文件，生成去除 .7z 后缀的文件名和带 .7z 后缀的文件名
        path_without_extension =  match.strip("\.7z")
        path_with_extension = f"{path_without_extension}.7z"  # 拼接 .7z 扩展名

        # 输出路径和大小调试信息
        print(f"检查文件: {path_without_extension}")
        print(f"检查文件: {path_with_extension}")

        # 获取文件大小
        size_without_extension = get_file_size(path_without_extension)
        size_with_extension = get_file_size(path_with_extension)

        #print(f"文件大小 (无扩展名): {size_without_extension}")
        #print(f"文件大小 (.7z): {size_with_extension}")

        # 确定文件0xoffset最小的文件
        if size_without_extension != -1 and size_without_extension < smallest_size:
            smallest_size = size_without_extension
            smallest_file = path_without_extension

        if size_with_extension != -1 and size_with_extension < smallest_size:
            smallest_size = size_with_extension
            smallest_file = path_with_extension

    # 输出最终结果
    smallist_with7z=get_file_size(path_with_extension)
    smallist_without7z=get_file_size(path_without_extension)
    #返回更大的那个文件
    if (smallist_with7z<smallist_without7z) & smallist_without7z in matching_files:
        print(f"\033[1;32m文件偏移表所在位置: {smallest_file}")
        return smallest_file
    else:
        print(f"\033[1;32m文件偏移表所在位置: {smallest_file+'.7z'}")
        return smallest_file+'.7z'

def find_files_offset_table(file_path):
    """
    针对已确定内含有偏移表的文件，使用strings自带的字符串偏移提取，从而寻找整个偏移表开头的位置
    """
    cmd = f"strings -t x -n 5 {file_path} | grep -E '\\.jpg|\\.png|\\.js|\\.css|\\.htm|\\.cer|\\.pem|\\.bin'"
    
    try:
        # 执行命令并获取结果，使用 universal_newlines=True 代替 text=True
        result = subprocess.check_output(cmd, shell=True, universal_newlines=True)
        print(result)
        # 按行分割结果并获取第一行
        first_line = result.splitlines()[0]
        
        # 提取偏移值（假设偏移和文件名之间有空格）
        offset = first_line.split()[0]
        
        # 打印带红色的偏移位置（使用 ANSI 转义代码控制颜色）
        print(f"\033[31m文件偏移表在【{file_path}】的【偏移({offset})】处\033[0m")
        return int(offset,16)
    except subprocess.CalledProcessError as e:
        print(f"执行命令时出错: {e}")
    except IndexError:
        print(f"未找到符合条件的结果，无法提取偏移。")

def extract_file_info(file_path, start_offset, endian='big'):
    """
    从指定的偏移量开始提取文件名和偏移信息，返回文件名及其偏移的键值对。
    在找到文件名后，继续向后找非零字符，然后对齐4字节，读取4字节内容作为偏移值。
    如果匹配到的文件名长度达到 0x100，说明已经是接下来的大片程序代码区域而非表格，则放弃继续匹配。
    参数:
    file_path: uImage镜像路径
    start_offset: 偏移表的开头
    endian: 端序，little或big
    """
    print(f"从偏移量 {hex(start_offset)} 减0x50处开始提取文件信息,增加容错率...")
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
                    adjusted_offset_hex = hex(adjusted_offset).lstrip("0x").upper() or "0"
                    # 如果文件名已经存在，则只保留最小的偏移量
                    if file_name in file_info:
                        file_info[file_name] = min(file_info[file_name], adjusted_offset)
                    else:
                        file_info[file_name] = adjusted_offset
                    print(f"文件名: {file_name}，相对文件系统偏移值: {hex(file_info[file_name]).upper()}")
                start = current_position + 4
            else:
                start = first_non_zero_start + 1
        else:
            start = first_non_zero_start + 1
    if not file_info:
        print("未找到任何文件名和偏移信息，可能文件格式不正确")
    file_info_str = {file_name: str(offset) for file_name, offset in file_info.items()}
    """
    min_offset_file = min(file_info, key=file_info.get) if file_info else None
    if min_offset_file:
        print(f"最小偏移量的文件名: {min_offset_file}, 偏移值: {hex(file_info[min_offset_file])}")
    """
    return file_info_str


def extract_offsets_from_start(data, path):
    # 用于存储从指定偏移开始后的十进制偏移数组
    decimal_offsets = []
    
    # 提取路径中的最后一个十六进制偏移地址
    start_hex_offset = re.search(r'/([A-Fa-f0-9]+)$', path.strip("\.7z"))

    if start_hex_offset:
        start_offset = int(start_hex_offset.group(1), 16)  # 使用 .group(1) 提取匹配的字符串
    else:
        print("未能提取有效的偏移地址。")
        return []
    
    # 正则表达式匹配每行的十进制偏移、十六进制值和描述
    pattern = re.compile(r'(\d+)\s+0x([A-Fa-f0-9]+)\s+(.*)')

    # 标志位，指示是否开始提取数据
    started = False

    # 遍历每一行，使用正则表达式匹配
    for line in data.splitlines():
        match = pattern.match(line)
        if match:
            decimal_offset = int(match.group(1))  # 十进制偏移
            hex_value = match.group(2)  # 十六进制值
            description = match.group(3)  # 描述部分
            # 如果已经开始提取数据，保存偏移并输出
            #if started:
            decimal_offsets.append(decimal_offset)
                # print(f"Hex Offset: {hex_value}, Description: {description}")
            # 一旦找到起始偏移值，开始提取后续的所有偏移
            #if hex_value.lower() == format(start_offset, 'x').lower():
            #    started = True

    return decimal_offsets



def rename_extracted_files(file_info, extracted_dir, filesystem_offset):  
    """ 
    根据给定的文件信息复制并重命名解压的文件，创建必要的文件夹结构。
    将文件复制到解压后的根目录之下并保留相对路径结构。

    参数：
    file_info : 文件信息，键为目标文件名，值为调整后的偏移值（十六进制字符串）。
    extracted_dir : 解压后的文件所在的目录。
    filesystem_offset : 文件系统的偏移值。
    """
    # 输出目录已经是解压后的目录，例如：vxfile_mw313rv4/_mw313rv4.bin.extracted
    print(f"正在尝试文件系统偏移为：{hex(filesystem_offset)}")
    print(f"开始复制文件并移动到结果目录...")
    false_count = 0
    for target_name, adjusted_offset_hex in file_info.items():
        try:
            # 将十六进制偏移值转换为整数并加上文件系统偏移量
            original_offset = int(adjusted_offset_hex) + filesystem_offset
            original_offset_hex = hex(original_offset).lstrip("0x").upper()  # 转换为八位十六进制字符串，去掉 0x 前缀，补足前导零
            #print(f"文件: {target_name}, 原始偏移值: {adjusted_offset_hex}, 计算后的偏移值: {original_offset_hex}")  #DEBUG用
        except ValueError as e:
            print(f"无效的偏移值: {adjusted_offset_hex}, 跳过该文件。错误: {e}")
            continue

        # 生成旧的文件路径（以偏移值为文件名，在解压后的目录中）
        old_file_path = os.path.join(extracted_dir, original_offset_hex)
        # 生成新的文件路径 (解压路径/result_vxworks_file/binwalk把内存偏移所在作为文件的名称)
        new_file_path = os.path.join(get_parent_directory(extracted_dir) + "/","result_vxworks_file",target_name.lstrip(os.sep))
        # 如果目标文件名包含路径，创建相应的目录结构
        target_directory = os.path.dirname(new_file_path)
        if target_directory and not os.path.exists(target_directory):
            os.makedirs(target_directory, exist_ok=True)

        # 复制文件
        if os.path.exists(old_file_path):
            try:
                print(f"已重命名文件 {old_file_path} 并复制到 {new_file_path}")
                shutil.copy2(old_file_path, new_file_path)
            except IOError as e:
                #print(f"复制文件失败: {old_file_path} 到 {new_file_path}，错误: {e}")
                pass
        else:
            false_count += 1
            print(f"文件 {old_file_path} 不存在，无法复制。")
        if(false_count>=10):
            print(f"文件系统偏移值{hex(filesystem_offset)}很可能不正确！正在换一个试试")
            return False
    return True



def main(file_path,fuzzymode):
    # 检查 binwalk 是否安装
    check_binwalk_installed()

    try:
        # 运行 binwalk 并获取解压目录
        binwalk_shell_output, target_directory, endian = run_binwalk_extract(file_path)
        # 首先尝试提取目标目录中的web资源文件名以寻找偏移表
        contained_filenames = extract_web_source_filenames(target_directory)        
        if contained_filenames and not fuzzymode:
            # 提取web资源文件名成功，那就使用精确的方案
            best_matching_file = find_binary_matches(target_directory, contained_filenames)
        else:
            # 提取web资源文件名失败，那就转而使用次精确的字符串匹配方案
            print("未找到任何web资源文件名，或用户指定使用fuzzy模糊搜索模式，可能固件没有http服务，转而使用次精确的字符串匹配方案")
            best_matching_file = fuzzy_search_file_contain_table(target_directory)
       
        infile_offset = find_files_offset_table(best_matching_file)
        file_info = extract_file_info(file_path, infile_offset, endian)
        maybe_filesystem_offsets = extract_offsets_from_start(binwalk_shell_output, best_matching_file)
        if maybe_filesystem_offsets:
            for testing_filesys_offset in maybe_filesystem_offsets:
                print(f"正在尝试以:{hex(testing_filesys_offset)}作为文件系统偏移")
                if(rename_extracted_files(file_info, target_directory, testing_filesys_offset)==True):
                    break

    except (ValueError, RuntimeError) as e:
        print(f"错误: {e}")
        
if __name__ == "__main__":
    # 检查是否请求了帮助信息
    if "-h" in sys.argv or "--help" in sys.argv or len(sys.argv) < 2:
        help_message = """
用法：
    python3 vxfile_extracter.py <bin 文件路径> [--fuzzmode]

选项：
    -h, --help      显示帮助信息
    --fuzzmode      使用模糊匹配模式处理文件

说明：
    该工具用于正确解压并恢复 vxworks 固件。默认情况下，优先使用精确匹配。如果指定 --fuzzmode 参数，将强制使用模糊匹配。
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
    fuzzymode = "--fuzzmode" in sys.argv

    # 调用主函数
    main(sys.argv[1], fuzzymode)
