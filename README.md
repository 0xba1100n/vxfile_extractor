# vxfile_extractor
[EN_readme](https://github.com/0xba1100n/vxfile_extractor/blob/main/README_EN.md)
## 版本更新
V2.0版本相较于V1.14这一概念验证版本，有了更兼容、更准确的匹配方案，欢迎提改进建议和issue
目前还是有不完美的地方，不能完全解出，比如tplink wr842nv7v9这些固件，接下来忙完手头上的事情周末立马看看是怎么回事。。。目前暂时没有头绪，因为就是对于某些固件，找不到文件偏移表，再看看吧
## 简介
一键提取vxwork固件内部文件，并依据binwalk分析结果和uboot镜像内文件名偏移表，正确恢复vxwork文件名
在日常分析固件的过程中，遇到不少设备是RTOS，其中目前见得最多的就是vxworks系统
binwalk尚未支持对vxworks系统固件的正确解包，而且已经停止更新很久了。直接用binwalk一把梭会遇到这种情况：全是莫名其妙的文件名
![1](https://github.com/user-attachments/assets/7aaf1cee-de63-4af5-b145-95eafdfd2d88)
这对于日常的分析而言，带来了麻烦。本来符号表缺失就有些麻烦了，现在连文件名都是"乱码"，分析下图类似于这类cgi路由自然是徒增几分困难
![2](https://github.com/user-attachments/assets/2f179233-c580-4f01-bef3-fd7c9b7fd512)

这个工具的功能是，在uImage中寻找偏移表，正确恢复文件名，并将恢复后的文件放入独立的结果文件夹；如果固件内部记录了路径，还会将文件放入正确的路径（否则将其放入一个默认文件夹，需要进一步分析）。
附上博客文章作为补充说明：https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/

# 使用方法
用法：
    python3 vxfile_extracter.py <bin 文件路径> [--fuzzymode]

选项：
    -h, --help      显示帮助信息
    --fuzzymode      使用模糊匹配模式处理文件

说明：
    该工具用于处理 bin 文件。默认情况下，优先使用精确匹配。如果指定 --fuzzymode 参数，将强制使用模糊匹配。

# 效果展示
该工具会一键解包、自动寻找偏移表位置、自动提取每个文件的偏移，并根据偏移表中记录的名称恢复文件名
![3](https://github.com/user-attachments/assets/6279fdca-8e35-4227-aea4-1621d7b0a329)
如果固件内部记录了路径，还会放入正确的路径
![4](https://github.com/user-attachments/assets/8f34b6ad-9655-4120-8e4d-3fc2efa180b6)
其中一个路径的文件如下
![5](https://github.com/user-attachments/assets/ae9c3f81-404e-46d1-a70d-e355e2ad12b8)
可以看到图片名称的恢复，也是能对上名字的
![image](https://github.com/user-attachments/assets/749b9416-5514-41da-ae15-5bff4ab66539)

# 测试用例
- Mercury水星mw313rV4固件
- tplink wr-855n固件
- tplink wr842nv3固件

# 注意事项
使用前请确保您的镜像能够通过binwalk正确解包，类似以下结果，这意味着固件没有加密，是正常的vxworks镜像
![384422042-7ef8cedf-0028-4f56-915f-35ddc8708229](https://github.com/user-attachments/assets/b2b05bd0-6176-4a75-b840-95c56fedb36e)

目前测试主要集中在vxworks 5.5及以下版本，这是最常见的版本，高版本暂时未遇到过。
某些固件的偏移表未记录路径，导致文件结构显得"扁平化"，需要进一步分析内部html文件引用以确定静态资源文件的正确位置。目前尚未找到合适的解决办法。
![image](https://github.com/user-attachments/assets/fb30a9dd-481d-4686-986c-20548dc40afd)

程序设计细节和恢复原理详见：https://bbs.kanxue.com/thread-284324.htm

如有问题欢迎提出issue，如果觉得好用请不吝⭐star，谢谢~
