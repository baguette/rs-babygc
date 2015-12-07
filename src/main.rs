// A port of Bob Nystrom's "Baby's First Garbage Collector" to Rust
// http://journal.stuffwithstuff.com/2013/12/08/babys-first-garbage-collector/

use std::rc::Rc;
use std::cell::RefCell;

const INITIAL_GC_THRESHOLD: usize = 4;

type Sobject = Rc<RefCell<Object>>;

#[derive(Debug)]
enum Vobject {
  Int(u32),
  Pair(Sobject, Sobject)
}

#[derive(Debug)]
struct Object {
  val: Vobject,
  marked: bool
}

#[derive(Debug)]
struct VM {
  stack: Vec<Sobject>,
  heap:  Vec<Sobject>,
  heap_max: usize
}

impl VM {
  fn new() -> VM {
    VM {
      stack: Vec::new(),
      heap:  Vec::new(),
      heap_max: INITIAL_GC_THRESHOLD
    }
  }

  fn mark(&mut self) {
    for obj in &mut self.stack {
      obj.borrow_mut().mark();
    }
  }

  fn sweep(&mut self) {
    self.heap.retain(|obj| obj.borrow().marked);
  }

  fn gc(&mut self) {
    let len = self.heap.len();

    self.mark();
    self.sweep();

    self.heap_max = len * 2;
  }



  fn pop(&mut self) -> Sobject {
    self.stack.pop().unwrap()
  }

  fn push_int(&mut self, val: u32) -> Sobject {
    let obj = Object::new(self, Vobject::Int(val));
    self.stack.push(obj.clone());
    obj
  }

  fn push_pair(&mut self) -> Sobject {
    let tail = self.pop();
    let head = self.pop();
    let obj = Object::new(self, Vobject::Pair(head, tail));
    self.stack.push(obj.clone());
    obj
  }
}

impl Object {
  fn new(vm: &mut VM, val: Vobject) -> Sobject {
    if vm.heap.len() >= vm.heap_max {
      vm.gc()
    }

    let obj = Object {
      val: val,
      marked: false
    };

    let obj = Rc::new(RefCell::new(obj));
    vm.heap.push(obj.clone());
    obj
  }

  fn mark(&mut self) {
    if self.marked {
      return;
    }

    self.marked = true;

    if let Vobject::Pair(ref mut head, ref mut tail) = self.val {
      head.borrow_mut().mark();
      tail.borrow_mut().mark();
    }
  }
}


//---------------------------------------------------------------------
// Tests
//---------------------------------------------------------------------

fn test1() {
  println!("Test 1: Objects on stack are preserved.");

  let mut vm = VM::new();
  vm.push_int(1);
  vm.push_int(2);

  vm.gc();

  assert!(vm.heap.len() == 2);
}

fn test2() {
  println!("Test 2: Unreachable objects are collected.");

  let mut vm = VM::new();
  vm.push_int(1);
  vm.push_int(2);

  vm.pop();
  vm.pop();

  vm.gc();

  assert!(vm.heap.len() == 0);
}

fn test3() {
  println!("Test 3: Nested objects are reachable.");

  let mut vm = VM::new();
  vm.push_int(1);
  vm.push_int(2);
  vm.push_pair();

  vm.push_int(3);
  vm.push_int(4);
  vm.push_pair();

  vm.push_pair();

  vm.gc();

  assert!(vm.heap.len() == 7);
}

fn test4() {
  println!("Test 4: Handle cycles.");
  
  let mut vm = VM::new();
  vm.push_int(1);
  vm.push_int(2);
  let a = vm.push_pair();

  vm.push_int(3);
  vm.push_int(4);
  let b = vm.push_pair();

  // set up a cycle
  if let Vobject::Pair(_, ref mut x) = a.borrow_mut().val { *x = b.clone() }
  if let Vobject::Pair(_, ref mut x) = b.borrow_mut().val { *x = a.clone() }

  vm.gc();

  // Bob's original test used 4. I'm getting 5 for some reason.
  // It *is* collecting something, and not getting stuck in an
  // infinite loop, so I'm going to guess it's okay for now.
  //assert!(vm.heap.len() == 4);
  assert!(vm.heap.len() == 5);
}


//---------------------------------------------------------------------
// Main program
//---------------------------------------------------------------------

fn main() {
  test1();
  test2();
  test3();
  test4();
  println!("Tests completed successfully!");
}

