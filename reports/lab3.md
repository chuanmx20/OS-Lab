# Lab3 

<center>钏茗喜 &nbsp 2020011035</center>

<HR>

## 实现思路
与之前的TASKMANAGER管理任务相关的工作不同，这次把相关task的管理抽象到了processor里管理，整体逻辑大抵相同。
### sys_get_time && sys_get_info
实现逻辑与前一次lab相同，设计地址转换的部分用`translated_refmut`来实现。task_info所需数据就存放在TCBinner里，同时TCB开放几个接口修改和访问。

### sys_mmap && sys_munmap
起初我照搬上一次的代码设计，发现无法通过测例。最后发现问题出在unmap的时候没有真正drop掉对应的area。所以我给MapArea添加了一个to_drop的标志位，当unmap的时候将其置为true，在unmap执行结束前只保留to_drop为0的area。

### spawn
仿照fork设计，不同的是，fork会拷贝一份用户空间和当前上下文，spawn只用基于elf元数据创建用户空间，新建上下文就行。

### stride
内容比较简单，就是给TCBinner新增两个字段，stride和priority，在执行的时候stride+=priority，每次选择stride最小的TCB执行（直接遍历找最小）。

## 思考题
1. 不是轮到p1，因为stride是8-bit uint，p2执行结束后stride会溢出，变成4，所以p2的stride最小。执行p2。
2. 当所有进程优先级都大于或等于2时，不同进程的stride值将以一种速率更新，以确保最大和最小stride值之间的差值始终在一定范围内。
3. 代码如下：
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            return Some(Ordering::Less);
        } else if self.0 > other.0 {
            return Some(Ordering::Greater);
        } else {
            return Some(Ordering::Equal);
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    > 看群里讨论，同学告知框架代码有误，修改了page_table.rs118行的框架代码

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    > 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

