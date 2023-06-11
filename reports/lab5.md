# Lab 5
<center>钏茗喜 &nbsp 2020011035</center>
<HR>

## 实现思路
检测死锁采用的是文档里提及的算法，整个lab5主要就是实现这个算法。
算法本身的内容并不复杂，复杂的是需要维护为了实现算法而引入的数组和矩阵。接下来分两部分说明。
### 死锁检测算法
算法根据work，finish，need，allocation四个数组来进行死锁检测。
1. 初始化work为available，finish为false，need为max-allocated，allocation为已分配的资源。
2. 从0开始遍历进程，如果finish为false且need小于等于work，则将work加上allocation，finish置为true，然后从0开始重新遍历。
3. 如果遍历结束后，finish全为true，则说明没有死锁，否则说明有死锁。
### 数据结构的维护
为了实现死锁检测，需要维护available，allocation，need这三个数据结构。以下是一些需要维护的场景：
**mutex:**
1. 创建新的mutex时，需要向available中添加一个元素，默认为1。同时需要向allocation和need添加新的一列，值为0。
2. 调用mutex_lock时，首先更新need，代表该进程需要的资源，然后调用死锁检测算法，如果检测到死锁，则返-0xDEAD，否则继续执行，将allocation中对应的值加1，available中对应的值减1，need中对应的值减1。
3. 在mutex_unlock中，将allocation中对应的值减1，available中对应的值加1。
**semaphore:**
1. 创建新的semaphore时，需要向available中添加一个元素，默认为初始值(resource_cnt)。同时需要向allocation和need添加新的一列，值为0。
2. 每次调用sem_down时，首先更新need，代表该进程需要的资源，然后调用死锁检测算法，如果检测到死锁，则返-0xDEAD，否则继续执行，将allocation中对应的值加1，available中对应的值减1，need中对应的值减1。
3. 在sem_up中，semaphore的count增加1以后，如果count小于等于0，说明有进程在等待，需要唤醒一个进程，唤醒的同时，将新线程在allocation中对应的值加need，available中对应的值减need，need对应的值设置为0，代表分配了资源。
**thread：**
1. 新建线程时，allocation和need新建一行或者如果线程id有对应的行就设为全0。
2. 线程exit时，清空allocation和need对应的行。
**new_process：**
1. 创建新的PCB时，需要对主线程(Thread 0)分配available，allocation和need的第0行。
2. 当fork一个新的进程时，需要将父进程available，allocation和need复制到新的一列。

## 思考题
> 1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

需要回收的资源包括：
1. 内存资源：该进程的所有内存空间，包括代码段、数据段、堆栈等。
2. 文件描述符：该进程打开的所有文件描述符。
3. 进程控制块(PCB)：该进程的PCB，包含了该进程的所有信息，如进程id、进程状态、进程优先级、进程的资源分配情况等。
4. 其他：消息队列、信号量、共享内存等。

其他线程的TCB可能在以下位置被引用：
1. 调度器
2. 同步机制，如semaphore的task_queue。
在进程结束的时候，所有线程的TCB都需要被回收，因为进程结束了，线程也就结束了，不需要再继续执行了。


> 2. 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？
```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```
两者的区别：Mutex1在unlock的时候先把locked置为false，然后再唤醒一个等待的线程；Mutex2在unlock的时候先唤醒一个等待的线程，然后再把locked置为false。
可能会导致的问题：
1. 对于Mutex1，在设置lock为false之后，在wake新的task之前，如果另一个线程获取了这个锁并且在新task开始运行前被唤醒，那么等待队列的任务就会在无锁的情况下运行，导致数据不一致。
2. 对于Mutex2，当等待任务队列中有任务时，锁会保持锁定。如果在添加到任务队列之后，在唤醒任务之前，另一个线程尝试lock，就会被阻塞，这一时刻没有任何线程实际上持有锁，这一定程度上会导致性能下降。


## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
> 无

1. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

> 无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

