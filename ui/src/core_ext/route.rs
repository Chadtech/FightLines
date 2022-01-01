use crate::route::Route;
use seed::prelude::Orders;

pub fn go_to_route<Msg: 'static>(orders: &mut impl Orders<Msg>, route: Route) {
    orders.request_url(route.to_url());
}
