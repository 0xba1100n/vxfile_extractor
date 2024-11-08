import re
import subprocess
import sys
import os
import shutil
import time

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
        print(f"执行命令: {' '.join(command)}")

        try:
            result = subprocess.run(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                universal_newlines=True,
                check=True
            )
            
            output = result.stdout
            print(f"binwalk 分析输出:\n{output}")
            return output, output_dir

        except subprocess.CalledProcessError as e:
            print(f"分析失败了，错误信息如下：")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            print("binwalk 可能没有正确安装或者发生了其他错误。请访问下列网址获取安装教程：")
            print("(此处添加网址)")
            sys.exit(1)

    # 输出目录不存在时，执行解包
    print("开始解包文件，并检查文件格式和加密状态...")
    command = ['binwalk', '-Me', '-C', output_dir, file_path]
    print(f"执行命令: {' '.join(command)}")
    
    try:
        result = subprocess.run(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            universal_newlines=True,
            check=True
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
        
        return output, output_dir

    except subprocess.CalledProcessError as e:
        print(f"解包失败了，错误信息如下：")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        print("binwalk 可能没有正确安装或者发生了其他错误。请访问下列网址获取安装教程：")
        print("(此处添加网址)")
        sys.exit(1)

def find_filesystem_offset(file_info):
    """
    从 binwalk 输出文本中提取 "IMG0 (VxWorks) header" 的下一行偏移位置。
    """
    print(f"开始查找文件系统偏移...")
    #print(f"提供的 binwalk 输出:\n{file_info}")

    pattern = re.compile(r"(\d+)\s+0x([0-9A-Fa-f]+)\s+IMG0 \(VxWorks\) header")
    matches = list(pattern.finditer(file_info))
    
    if not matches:
        print("未找到 'IMG0 (VxWorks) header' 相关信息，可能不是标准的 vxworks 镜像。")
        sys.exit(1)
    
    last_match = matches[-1]
    last_img0_start = last_match.start()
    last_img0_end = last_match.end()
    
    try:
        previous_line = file_info[:last_img0_start].splitlines()[-1].strip()
        previous_offset_match = re.match(r"\d+\s+0x([0-9A-Fa-f]+)", previous_line)
        following_line = file_info[last_img0_end:].splitlines()[1].strip()
        following1_offset_match = re.match(r"\d+\s+0x([0-9A-Fa-f]+)", following_line)
        f2_line = file_info[last_img0_end:].splitlines()[2].strip()
        following2_offset_match = re.match(r"\d+\s+0x([0-9A-Fa-f]+)", f2_line)
        f3_line = file_info[last_img0_end:].splitlines()[3].strip()
        following3_offset_match = re.match(r"\d+\s+0x([0-9A-Fa-f]+)", f3_line)
        if previous_offset_match and following1_offset_match and following2_offset_match and following3_offset_match:
            result = {
                "uboot_img_file_name": previous_offset_match.group(1),
                "file_system_offset1": f"0x{following1_offset_match.group(1)}",
                "file_system_offset2": f"0x{following2_offset_match.group(1)}",
                "file_system_offset3": f"0x{following3_offset_match.group(1)}"
            }
            print(f"找到的文件系统偏移结果: {result}")
            return result
        else:
            print("偏移信息不完整，可能不是标准的 vxworks 镜像")
            sys.exit(1)
    except IndexError:
        print("在提取偏移信息时遇到错误，可能是数据格式不正确")
        sys.exit(1)


def find_start_marker(file_path, start_position=0):
    """
    从文件指定位置开始扫描，找到连续指定数量的0x00字节后的第一个非0x00字节位置。
    """
    zero_threshold = 128
    with open(file_path, 'rb') as f:
        data = f.read()

    print(f"{file_path} 文件大小: {len(data)} 字节")

    start = start_position
    consecutive_zeros = 0
    found = False

    while start < len(data):
        # 检查当前字节是否为0x00
        if data[start] == 0x00:
            consecutive_zeros += 1
        else:
            consecutive_zeros = 0

        # 如果找到了连续指定数量的0x00字节
        if consecutive_zeros >= zero_threshold:
            found = True
            print(f"找到了符合条件的连续 {zero_threshold} 个0x00字节，位置从 {start - zero_threshold + 1} 到 {start}")
            break

        start += 1

    if found:
        # 从找到的连续 0x00 区域的结束位置继续寻找第一个非 0x00 字节
        for pos in range(start + 1, len(data)):
            if data[pos] != 0x00:
                print(f"找到连续 {zero_threshold} 个0x00后的第一个非0x00字节位置: {pos}")
                return pos

    print("寻找文件名偏移表失败，未找到符合条件的位置")
    return 0


def extract_file_info(file_path, start_offset, endian='big'):
    """
    从指定的偏移量开始提取文件名和偏移信息，返回文件名及其偏移的键值对。
    在找到文件名后，继续向后找非零字符，然后对齐4字节，读取4字节内容作为偏移值。
    如果匹配到的文件名长度达到 0x100，说明已经是接下来的大片程序代码区域而非表格，则放弃继续匹配。
    """
    print(f"从偏移量 {hex(start_offset)} 开始提取文件信息...")
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
                    print(f"文件名: {file_name}，对应偏移值: {hex(file_info[file_name]).upper()}")
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

def rename_extracted_files(file_info, output_dir, filesystem_offset):  
    """ 
    根据给定的文件信息复制并重命名解压的文件，创建必要的文件夹结构。
    将文件复制到解压后的根目录之下并保留相对路径结构。

    参数：
    file_info (dict): 文件信息，键为目标文件名，值为调整后的偏移值（十六进制字符串）。
    output_dir (str): 解压后的文件所在的目录。
    filesystem_offset (int): 文件系统的偏移值。
    """
    # 输出目录已经是解压后的目录，例如：vxfile_mw313rv4/_mw313rv4.bin.extracted
    extracted_dir = output_dir
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
        new_file_path = os.path.join(extracted_dir.split("/")[0] + "/","result_vxworks_file",target_name.lstrip(os.sep))
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

def main(file_path):
    print("欢迎使用 binwalk 解包工具！")
    check_binwalk_installed()
    print(f"准备解包文件: {file_path}\n")
    output, output_dir = run_binwalk_extract(file_path)
    filesystem_info = find_filesystem_offset(output)
    uboot_img_file_path = os.path.join(output_dir, filesystem_info["uboot_img_file_name"])  
    uboot_img_file_path += ".7z"  # not just the file, it is a .7z file
    if not os.path.exists(uboot_img_file_path):
        print(f"U-Boot 镜像文件 {uboot_img_file_path} 不存在，可能解包失败。")
        sys.exit(1)
    file_info = None
    current_position = 0  # 初始从文件头开始查找
    while True:
        # Step 3: Find start marker from the current position
        start_marker = find_start_marker(uboot_img_file_path, current_position)
        file_info = extract_file_info(uboot_img_file_path, start_marker, "big")
        if file_info:
            break
        current_position = start_marker + 0x100  # 跳过一段区域，避免卡在无效区域
        print(f"未找到文件信息，重新尝试查找... (当前偏移位置: {current_position})")
        if current_position >= os.path.getsize(uboot_img_file_path):
            print("已经到达文件末尾，仍未找到有效的文件信息。程序退出。")
            sys.exit(1)
    filesystem_offset1 = int(filesystem_info["file_system_offset1"], 16)
    filesystem_offset2 = int(filesystem_info["file_system_offset2"], 16)
    filesystem_offset3 = int(filesystem_info["file_system_offset3"], 16)
    if(rename_extracted_files(file_info, output_dir,filesystem_offset1)==False):
        if(rename_extracted_files(file_info, output_dir,filesystem_offset2)==False):
            if(rename_extracted_files(file_info, output_dir,filesystem_offset3)==False):
                print("尝试高度可能的文件偏移都不行，请自行寻找...")
                return False
    result_path_relative = os.path.join(output_dir.split("/")[0], "result_vxworks_file")
    result_path_absolute = os.path.abspath(os.path.join(os.getcwd(), result_path_relative))
    print(f"\033[1;32m解析成功！请到 {result_path_absolute} 查看解析的文件\033[0m")
    return True
if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("用法：python vxfile_extracter.py <bin 文件路径>")
        sys.exit(1)
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
    main(sys.argv[1])


