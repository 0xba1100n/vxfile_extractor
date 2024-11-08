# vxfile_extractor
一键提取vxwork固件内部文件，并依据binwalk分析结果和uboot镜像内文件名偏移表，正确恢复vxwork文件名
在日常分析固件的过程中，遇到不少设备是RTOS，其中目前见得最多的就是vxworks系统
binwalk尚未支持对vxworks系统固件的正确解包，而且已经停止更新很久了。直接用binwalk一把梭会遇到这种情况：全是莫名其妙的文件名
![1](https://github.com/user-attachments/assets/7aaf1cee-de63-4af5-b145-95eafdfd2d88)
这对于日常的分析而言会带来麻烦，本来符号表缺失就有些麻烦了，现在连文件名都是"乱码"，分析下图类似于这类cgi路由自然是徒增几分困难
![2](https://github.com/user-attachments/assets/2f179233-c580-4f01-bef3-fd7c9b7fd512)

这个工具做的就是在uImage找偏移表，把文件名正确恢复，并放入独立的结果文件夹；如果固件内部有记载路径，还会放入正确的路径（否则会放到一个文件夹，需要进一步的分析）

# 使用方法
python3 vxfile_extracter.py <bin 文件路径>

# 效果展示
该工具会一键解包、自动寻找偏移表位置、自动提取每个文件的偏移，并为每个文件按偏移表里的记载名称进行文件名恢复
![3](https://github.com/user-attachments/assets/6279fdca-8e35-4227-aea4-1621d7b0a329)
如果在固件内部有正确的路径记载，还会放入正确的路径
![4](https://github.com/user-attachments/assets/8f34b6ad-9655-4120-8e4d-3fc2efa180b6)
其中一个路径的文件如下
![5](https://github.com/user-attachments/assets/ae9c3f81-404e-46d1-a70d-e355e2ad12b8)

# 测试用例
mw313rV4固件
tplink wr-855n固件

# 注意事项
基本上只在vxworks5.5以下测过，这也是最最常见的版本，而更高的版本目前没有遇到过
有些固件的偏移表没有记录路径，会显得“扁平化”，需要进一步分析内部htm文件引用以将静态资源文件放入正确位置，目前没有想到合适的办法
![image](https://github.com/user-attachments/assets/fb30a9dd-481d-4686-986c-20548dc40afd)
有问题欢迎issue，如果好用请不吝啬您的⭐star，谢谢~
