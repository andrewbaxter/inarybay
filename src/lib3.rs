use std::collections::VecDeque;
use gc::{
    GcCellRefMut,
    Gc,
    GcCell,
};
use proc_macro2::TokenStream;
use quote::quote;

struct ReadState {}

struct WriteState {}

#[derive(gc::Trace, gc::Finalize)]
struct NodeData<T: SubnodeTrait> {
    prev: i8,
    next: Vec<Node>,
    sub: T,
}

trait SubnodeTrait: gc::Trace + gc::Finalize {
    fn generate_read(&mut self, state: &mut ReadState, out: &mut Vec<TokenStream>);
    fn generate_write(&mut self, state: &mut WriteState, out: &mut Vec<TokenStream>);
}

trait NodeTrait: gc::Trace + gc::Finalize {
    fn generate_read(&mut self, stack: &mut Vec<Node>, state: &mut ReadState, out: &mut Vec<TokenStream>);
    fn generate_write(&mut self, stack: &mut Vec<Node>, state: &mut WriteState, out: &mut Vec<TokenStream>);
    fn dec_prev(&mut self, self2: &Node, stack: &mut Vec<Node>);
}

type Node = Gc<GcCell<dyn NodeTrait>>;

impl<T: SubnodeTrait> NodeTrait for NodeData<T> {
    fn dec_prev(&mut self, self2: &Node, stack: &mut Vec<Node>) {
        self.prev -= 1;
        if self.prev < 0 {
            panic!();
        }
        if self.prev == 0 {
            stack.push(self2.clone());
        }
    }

    fn generate_read(&mut self, stack: &mut Vec<Node>, state: &mut ReadState, out: &mut Vec<TokenStream>) {
        self.sub.generate_read(state, out);
        for n in self.next {
            n.borrow_mut().dec_prev(&n, stack);
        }
    }

    fn generate_write(&mut self, stack: &mut Vec<Node>, state: &mut WriteState, out: &mut Vec<TokenStream>) {
        self.sub.generate_write(state, out);
        for n in self.next {
            n.borrow_mut().dec_prev(&n, stack);
        }
    }
}

type Bits = usize;

#[derive(gc::Trace, gc::Finalize)]
struct Frame {
    size: Bits,
}

impl SubnodeTrait for Frame {
    fn generate_read(&mut self, state: &mut ReadState, out: &mut Vec<TokenStream>) {
        let size = self.size;
        out.push(quote!{
            buf.write(source.read($size));
        });
    }

    fn generate_write(&mut self, state: &mut WriteState, out: &mut Vec<TokenStream>) {
        let size = self.size;
        out.push(quote!{
            buf.reserve($size);
        })
    }
}

#[derive(gc::Trace, gc::Finalize)]
struct NumNode {
    size: usize,
    le: bool,
    signed: bool,
    rusttype: String,
    location: Location,
}

impl SubnodeTrait for NumNode {
    fn generate_read(&mut self, state: &mut ReadState, out: &mut Vec<TokenStream>) {
        let bit_size = self.size % 8;
        let byte_size = self.size / 8;
        let offset = self.offset.unwrap();
        let bit_offset = offset % 8;
        let byte_offset = offset / 8;
        if bit_size > 0 {
            let mut mask = 0u8;
            for i in 0 .. bit_size {
                mask = mask << 1;
                mask |= 1;
            }
            out.push(quote!{
                $rust =((buf[$byte_offset] >> $bit_offset) & $mask) as $rusttype;
            });
        } else {
            out.push(quote!{
                $rust = $rusttype:: from(buf[$byte_offset..$byte_offset + $byte_size]);
            });
        }
    }

    fn generate_write(
        &mut self,
        stack: &mut Vec<Box<Self>>,
        state: &mut WriteState,
        out_write: &mut Vec<TokenStream>,
    ) {
        todo!()
    }
}

fn generate(root: X) {
    let out_read = vec![];
    let mut stack = Vec::new();
    let mut state = ReadState::new();
    while let Some(e) = stack.pop() {
        e.generate(&mut stack, &mut state, &mut out_read);
    }
}

fn test() { }
