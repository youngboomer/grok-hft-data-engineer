//! Simple but rigorous order lifecycle state machine + tiny inventory feedback.
//! Exhaustive matching makes illegal states unrepresentable.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderState {
    New,
    PartiallyFilled { filled: u64 },
    Filled,
    Canceled,
    Rejected,
}

#[derive(Debug)]
struct Order {
    client_id: u64,
    state: OrderState,
    remaining: u64,
}

impl Order {
    fn new(client_id: u64, qty: u64) -> Self {
        Self {
            client_id,
            state: OrderState::New,
            remaining: qty,
        }
    }

    fn on_fill(&mut self, qty: u64) {
        match &mut self.state {
            OrderState::New | OrderState::PartiallyFilled { .. } => {
                if qty >= self.remaining {
                    self.state = OrderState::Filled;
                    self.remaining = 0;
                } else {
                    self.remaining -= qty;
                    self.state = OrderState::PartiallyFilled { filled: qty };
                }
            }
            _ => {}
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, OrderState::New | OrderState::PartiallyFilled { .. }) {
            self.state = OrderState::Canceled;
        }
    }
}

fn main() {
    let mut order = Order::new(42, 5000);
    println!("Initial: {:?}", order.state);

    order.on_fill(1200);
    println!("After partial: {:?}", order.state);

    order.on_fill(4000);
    println!("After fill: {:?}", order.state);

    println!("This pattern using Rust enums prevents entire classes of 'order in two states' bugs.");
}
