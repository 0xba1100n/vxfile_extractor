# vxfile_extractor
一键提取vxwork固件内部文件，并依据binwalk分析结果和uboot镜像内文件名偏移表，正确恢复vxwork文件名
在日常分析固件的过程中，遇到不少设备是RTOS，其中目前见得最多的就是vxworks系统
binwalk尚未支持对vxworks系统固件的正确解包，而且已经停止更新很久了。直接用binwalk一把梭会遇到这种情况：全是莫名其妙的文件名
![image](https://github.com/user-attachments/assets/e1f3a9ea-485a-4eb7-9628-42db0ebd0d49)
这对于日常的分析而言会带来麻烦，本来符号表缺失就有些麻烦了，现在连文件名都是"乱码"，分析类似于这类cgi路由自然是徒增几分困难
![image](https://github.com/user-attachments/assets/5ae400a9-2fff-4054-aff7-97ab765b76b9)

这个工具做的就是在uImage找偏移表，把文件名正确恢复，并放入独立的结果文件夹；如果固件内部有记载路径，还会放入正确的路径（否则会放到一个文件夹，需要进一步的分析）
# 效果展示

![image](https://github.com/user-attachments/assets/abd228ae-b9de-4c75-9498-b366eb72ee29)

![image](https://github.com/user-attachments/assets/f81f9ef2-f16c-4202-9091-df3d91477108)
![image](https://github.com/user-attachments/assets/a35d0c68-e803-4e56-a671-a878a2196bc1)
