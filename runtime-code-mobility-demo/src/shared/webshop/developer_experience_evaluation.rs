use crate::shared::webshop::order_history::Order;
use std::collections::HashMap;

pub fn get_most_bought_product(orders: &Vec<Order>) -> Option<MostBoughtProduct> {
    let mut quantities: HashMap<String, u32> = HashMap::new();

    for order in orders {
        for product in &order.products {
            *quantities.entry(product.name.clone()).or_insert(0) += product.quantity;
        }
    }

    quantities
        .into_iter()
        .max_by_key(|&(_, qty)| qty)
        .map(|(name, total_quantity)| MostBoughtProduct {
            name,
            total_quantity,
        })
}

pub fn get_most_invested_product(orders: &Vec<Order>) -> Option<MostInvestedProduct> {
    let mut amounts: HashMap<String, f64> = HashMap::new();

    for order in orders {
        for product in &order.products {
            *amounts.entry(product.name.clone()).or_insert(0.0) +=
                product.quantity as f64 * product.price;
        }
    }

    amounts
        .into_iter()
        .max_by(|&(_, amount_a), &(_, amount_b)| amount_a.partial_cmp(&amount_b).unwrap())
        .map(|(name, total_amount)| MostInvestedProduct { name, total_amount })
}

pub struct MostBoughtProduct {
    pub name: String,
    pub total_quantity: u32,
}

pub struct MostInvestedProduct {
    pub name: String,
    pub total_amount: f64,
}
