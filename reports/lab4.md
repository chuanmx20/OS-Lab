# Lab 4
<center>钏茗喜 &nbsp 2020011035</center>
<HR>

## 实现思路
### link
link实现过程比较简单，首先是几个合法性检查：
1. 检查old_name是否对应一个文件
2. 检查new_name是否已经存在
3. old_name和new_name是否不同
然后在ROOT_INODE下找到old_name对应的inode，创建一个新的DirEntry，写到ROOT_INODE下即可。
### unlink
相较于link，unlink需要做的工作更多一些，首先是合法性检查，即name是否对应一个文件。
然后根据name找到对应的dirent，如果找不到则报错，如果找到了，就将其从ROOT_INODE下删除。
然后根据dirent的id找到对应的位置root_inode对应的entry项为0。接下来检查是否还有其他的dirent指向这个inode，如果没有，就将其释放。
### fstat
fstat需要获取文件的以下几个信息（已忽略不需要考虑的地方）：
1. inode_id
2. StatMode
3. nlinks
首先对fd进行合法性检查，看当前文件的fd_table中是否有对应的fd，如果没有就报错。
根据fd从fd_table中找到File Desc，拓展框架代码中的File trait，添加一个inode_id的方法，用于获取当前文件的inode_id。
向RootINode里实现这个接口，使用inode的block_id和block_offset得到inode_id。
StatMode的话就实现一个从inode_id到inode对象的方法，即根据inode_id获取block_id和block_offset，通过这两个参数创建新的inode对象。然后使用is_dir方法判断是文件还是目录，然后根据is_dir的结果创建对应的StatMode。
nlinks就遍历一遍ROOT_INODE下的dirent，找到inode_id对应的dirent的个数即可。
## 思考题
> Q：在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

A: 由于efs中所有文件都在根目录中，root inode就是根目录的inode，它的内容包含了根目录下的所有文件的dirent，在启动任何一个程序时，都会根据root inode中的dirent找到对应的inode，然后根据inode中的内容找到对应的数据块，从而读取文件内容。如果root inode中的内容损坏了，那么就会导致无法找到任何文件，因此无法启动任何程序。
## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

> 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

> 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

