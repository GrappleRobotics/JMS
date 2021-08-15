use handlebars::{handlebars_helper, Handlebars};
use ipnetwork::Ipv4Network;
use serde_json::json;

fn v4_network(n: &str) -> Ipv4Network {
  n.parse().unwrap()
}

handlebars_helper!(ip: |nw: str| json!(v4_network(nw).ip().to_string()));
handlebars_helper!(network: |nw: str| json!(v4_network(nw).nth(0).unwrap().to_string()));
handlebars_helper!(netmask: |nw: str| json!(v4_network(nw).mask().to_string()));

pub fn setup(hbars: &mut Handlebars) {
  hbars.register_helper("netmask", Box::new(netmask));
  hbars.register_helper("network", Box::new(network));
  hbars.register_helper("ip", Box::new(ip));
}
