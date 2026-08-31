use circular_buffer::FixedCircularBuffer;

fn main() {
    //cargo add circular_buffer
    let mut buf = FixedCircularBuffer::<&str, 5>::new();

    buf.push_back("Hello");
    buf.push_back(" World");
    buf.push_back(",Rust!");
    for i in buf{
        println!("The value is: {}",i)
    }
}
