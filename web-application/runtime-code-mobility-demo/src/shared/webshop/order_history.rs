#[derive(Clone)]
pub struct OrderedProduct {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

/// @mobile
impl OrderedProduct {
    pub fn new(id: String, name: String, price: f64, quantity: u32) -> Self {
        Self {
            id,
            name,
            price,
            quantity,
        }
    }
}

#[derive(Clone)]
pub struct Order {
    pub id: String,
    pub products: Vec<OrderedProduct>,
    pub total: f64,
    pub starred: bool,
    pub archived: bool,
}

/// @mobile
impl Order {
    pub fn new(id: String, products: Vec<OrderedProduct>) -> Self {
        let total = products
            .iter()
            .fold(0.0, |acc, p| acc + (p.price * p.quantity as f64));
        Self {
            id,
            products,
            total,
            starred: false,
            archived: false,
        }
    }

    pub fn toggle_archive(&mut self) {
        self.archived = !self.archived;
    }

    pub fn toggle_starred(&mut self) {
        self.starred = !self.starred;
    }
}

#[derive(Clone)]
pub struct OrderManager {
    pub orders: Vec<Order>,
}

/// @mobile
impl OrderManager {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn add(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn get(&self, order_id: String) -> Order {
        self.orders
            .iter()
            .find(|o| o.id == order_id)
            .expect("Order not found")
            .clone()
    }

    pub fn get_mut(&mut self, order_id: String) -> &mut Order {
        self.orders
            .iter_mut()
            .find(|o| o.id == order_id)
            .expect("Order not found")
    }

    pub fn filter(&self, order_filter_options: OrderFilterOptions) -> Vec<Order> {
        let mut filtered_orders = self.orders.clone();
        if let Some(starred) = order_filter_options.starred {
            filtered_orders.retain(|o| o.starred == starred);
        }
        if let Some(archived) = order_filter_options.archived {
            filtered_orders.retain(|o| o.archived == archived);
        }
        filtered_orders
    }

    pub fn toggle_archive(&mut self, order_id: String) {
        let order = self.get_mut(order_id);
        order.toggle_archive();
    }

    pub fn toggle_starred(&mut self, order_id: String) {
        let order = self.get_mut(order_id);
        order.toggle_starred();
    }

    pub fn get_total_expenses(&self) -> f64 {
        get_total_expenses(&self.orders)
    }
}

pub struct OrderFilterOptions {
    pub starred: Option<bool>,
    pub archived: Option<bool>,
}

pub fn get_total_expenses(orders: &[Order]) -> f64 {
    orders.iter().fold(0.0, |acc, o| acc + o.total)
}
