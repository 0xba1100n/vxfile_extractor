# vxfile_extractor
[中文介绍](https://github.com/0xba1100n/vxfile_extractor/blob/main/README.md)

## Introduction
Extract files from vxworks firmware with one click, and correctly restore vxworks filenames based on binwalk analysis results and the file name offset table in the uboot image.
In the process of analyzing firmware, many devices are RTOS-based, and the most common system encountered is vxworks.
Binwalk does not yet support correct unpacking of vxworks firmware and has not been updated for a long time. Using binwalk directly might lead to issues such as nonsensical filenames like the one below:
![1](https://github.com/user-attachments/assets/7aaf1cee-de63-4af5-b145-95eafdfd2d88)
This can cause problems in daily analysis. Missing symbol tables are already troublesome, and now with filenames in "garbled" characters, analyzing routes like those in the image below can become significantly harder:
![2](https://github.com/user-attachments/assets/2f179233-c580-4f01-bef3-fd7c9b7fd512)

This tool looks for the offset table in uImage, correctly restores filenames, and saves them into a separate result folder. If the firmware contains recorded paths, it will place them into the correct directory (otherwise, they will be placed into a default folder for further analysis).
Here's a blog post for additional clarification: https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/

# Usage
Usage:
    python3 vxfile_extracter.py <bin file path> [--fuzzymode]

Options:
    -h, --help      Show help information
    --fuzzymode      Use fuzzy matching mode to process files

Description:
    This tool is used to process bin files. By default, it uses exact matching. If the --fuzzymode parameter is specified, it will force fuzzy matching.

# Example Output
The tool will unpack, automatically locate the offset table, extract each file's offset, and restore filenames based on the names in the offset table.
![3](https://github.com/user-attachments/assets/6279fdca-8e35-4227-aea4-1621d7b0a329)
If the firmware contains correct paths, the files will be placed in the right locations.
![4](https://github.com/user-attachments/assets/8f34b6ad-9655-4120-8e4d-3fc2efa180b6)
One of the file paths is shown below:
![5](https://github.com/user-attachments/assets/ae9c3f81-404e-46d1-a70d-e355e2ad12b8)
The filename restoration can also match correctly.
![image](https://github.com/user-attachments/assets/749b9416-5514-41da-ae15-5bff4ab66539)

# Test Cases
- Mercury MW313R V4 firmware
- TP-Link WR-855N firmware
- TP-Link WR842N V3 firmware
- TP-Link Archer C80 v1v2 firmware

# Notes
Before use, please ensure that your image can be unpacked by binwalk and shows results like the one below. This means it is a valid, unencrypted vxworks image:
![384422042-7ef8cedf-0028-4f56-915f-35ddc8708229](https://github.com/user-attachments/assets/b2b05bd0-6176-4a75-b840-95c56fedb36e)

Testing has been primarily focused on vxworks 5.5 and earlier versions, which are the most common. Higher versions have not been encountered.
Some firmware offset tables do not record paths, causing the structure to appear "flattened". Further analysis of internal HTML file references is needed to correctly place static resource files, and currently, no suitable solution has been found.
![image](https://github.com/user-attachments/assets/fb30a9dd-481d-4686-986c-20548dc40afd)

For program design details and restoration principles, see: https://bbs.kanxue.com/thread-284324.htm

If you have any questions, feel free to submit an issue. If you find this useful, please don’t hesitate to ⭐star it. Thanks!
