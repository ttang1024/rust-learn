#[derive(Debug)]
struct Node {
    val: i32,
    // Option<T> 有两个变量：
    // None，表明失败或缺少值
    // Some(value)，元组结构体，封装了一个 T 类型的值 value
    // 使用 Box<T> 将数据存储在堆上
    next: Option<Box<Node>>, 
}

impl Node {
    fn new(val: i32) -> Box<Node> {
        Box::new(Node { val, next: None })
    }

    fn link(&mut self, node: Box<Node>) {
        self.next = Some(node);
    }
}

#[derive(Debug)]
struct BoxedLinkedList {
    head: Option<Box<Node>>,
}

impl BoxedLinkedList {
    fn new() -> BoxedLinkedList {
        BoxedLinkedList { head: None }
    }

    fn display(&self) {
        println!("{:?}", self);
    }

    fn push_front(&mut self, val: i32) {
        let mut new_node = Node::new(val);
        if let Some(node) = self.head.take() {
            new_node.link(node);
        }
        self.head = Some(new_node);
    }

    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    fn pop_front(&mut self) -> Option<i32> {
        // 获取头指针的包裹的值；生成一个新的Option；
        match self.head.take() {
            None => None, // 为空的话直接返回None；
            Some(node) => {
                // 注意此时拥有 node 的所有权；
                self.head = node.next; // Box自动解引用，然后将node的next的所有权转移给变量head；
                Some(node.val)
            }
        }
    }

    fn push_back(&mut self, value: i32) {
        let new_node = Node::new(value);
        match self.head.as_mut() {
            None => self.head = Some(new_node),
            Some(mut curr) => {
                while curr.next.is_some() {
                    curr = curr.next.as_mut().unwrap();
                }
                curr.link(new_node);
            }
        }
    }

    fn pop_back(&mut self) -> Option<i32> {
        match self.head.as_mut() {
            None => None,
            Some(mut curr) => {
                while curr.next.is_some() && curr.next.as_ref().unwrap().next.is_some() {
                    curr = curr.next.as_mut().unwrap();
                }
                match curr.next {
                    Some(_) => Some(curr.next.take().unwrap().val),
                    None => Some(self.head.take().unwrap().val),
                }
            }
        }
    }

    fn insert(&mut self, value: i32) {
        let mut new_node = Node::new(value);
        match self.head.as_mut() {
            None => self.head = Some(new_node), // 链表为空;
            Some(mut curr) => {
                // 插入值小于头节点的值；此时 curr == head;
                if value <= curr.val {
                    self.push_front(value); // 直接调用头部插入函数；
                } else {
                    // 遍历寻找插入位置
                    while curr.next.is_some() && curr.next.as_ref().unwrap().val < value {
                        curr = curr.next.as_mut().unwrap();
                    }

                    // 如果插入的位置不为尾节点之后；取出下一个节点并获取其所有权并link到新节点之后；
                    if let Some(node) = curr.next.take() {
                        new_node.link(node);
                    }
                    // 当前节点的next指向新节点；
                    curr.next = Some(new_node);
                }
            }
        }
    }
    fn delete(&mut self, value: i32) -> bool {
        match self.head.as_mut() {
            None => false, // 链表为空；
            Some(mut curr) => {
                // 待删除节点为头节点的情况; 此时 curr == head;
                if curr.val == value {
                    // 也可以调用 self.pop_front();这里故意给出不同实现;
                    self.head = self.head.take().unwrap().next;
                    true
                } else {
                    // 从头节点的下一个节点开始寻找待删除节点，直到尾节点或找到为止；
                    while curr.next.is_some() && curr.next.as_ref().unwrap().val != value {
                        curr = curr.next.as_mut().unwrap();
                    }

                    // 当前curr的下一个节点的值等于value，或者curr为尾节点；
                    match curr.next.take() {
                        None => false, // curr已经是尾节点，这时相当于没有找到符合条件的节点;
                        Some(node) => {
                            // 获得下一个节点，并将其next的值的所有权转移到curr.next，删除成功；
                            curr.next = node.next;
                            true
                        }
                    }
                }
            }
        }
    }
    fn traverse(&self) {
        let mut curr = self.head.as_ref();
        while curr.is_some() {
            print!("{:?}\t", curr.unwrap().val);
            curr = curr.unwrap().next.as_ref();
        }
    }
    fn reverse(&mut self) {
        // 如果链表为空或只有一个节点，直接返回；
        if self.is_empty() || self.head.as_ref().unwrap().next.is_none() {
            return;
        }

        // 获取头节点的下一个节点的值的所有权；left指向剩余未反转的链表的第一个节点；
        // 注意此处因为take方法的调用，self.head.next的值已经为None了；故不需要再显式写下面这行代码了：
        // self.head.as_mut().unwrap().next = None;
        let mut left = self.head.as_mut().unwrap().next.take();
        while left.is_some() {
            let mut t = left.take().unwrap(); // 从Option中取出节点赋值给临时变量t；
            left = t.next; // left向后移动一个；
            t.next = self.head.take(); // t.next指向头节点
            self.head = Some(t); // 头节点指针指向t；
        }
    }
}

fn main() {
    let mut list = BoxedLinkedList::new();
    for v in vec![4, 2, 8, 1, 0, 3, 9] {
        list.insert(v);
    }

    list.display();

    println!();
    list.traverse();

    list.reverse();

    println!("\n\nafter reverse:");
    list.display();

    println!();
    list.traverse();
}
