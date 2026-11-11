# Examples

Every example is a program you can run with `cargo run --example <name>`. It reads the account from
two variables, as the crate does:

```sh
export PICQER_SUBDOMAIN=example
export PICQER_API_KEY=...
cargo run --example list_orders
```

Use an account meant for development. The table says what an example does to the account. Examples
that need an id of your account take it as an argument, and each one starts with the command to run it.

| Example | Shows | Account |
|---|---|---|
| `list_orders` | Listing orders with a filter, one page and all pages | reads |
| `errors_and_rate_limit` | Matching on error codes, the rate limit, retry and throttle | reads |
| `reference_data` | Warehouses, VAT groups, tags, pricelists, users, templates, packagings, shipping providers, location types, packing stations, order fields, stats and webshop orders | reads |
| `create_order` | Making an order, processing it and waiting for the status | writes |
| `order_actions` | Order lines, tags, allocation, prioritising, cancelling and undoing it | writes |
| `products` | Making, finding, changing and inactivating a product | writes |
| `product_details` | Price history, expected deliveries and warehouse settings | writes |
| `stock` | Adding, removing and moving stock, and the stock history | writes |
| `custom_fields` | Values of product, order and customer fields | writes |
| `compositions` | Parts of a composition, its compositions and producing | writes |
| `images` | Adding and removing a product image | writes |
| `customers` | Customers with an address and a tag | writes, cleans up |
| `picklist_workflow` | Picking, shipping and closing a picklist | writes |
| `picklist_batches` | Open batches, adding and removing a picklist, presets | writes, cleans up |
| `picking_containers` | Making, renaming, linking and deleting a picking container | writes, cleans up |
| `locations` | Locations, linking one to a product | writes, cleans up |
| `backorders` | Listing backorders and processing them | reads, writes with `process` |
| `purchasing` | A supplier, a purchase order and a receipt | writes, cleans up |
| `returns` | A return, its logbook and deleting it | writes, cleans up |
| `tasks_and_comments` | Tasks and comments on an order | writes |
| `tags_and_packagings` | Making and changing tags and packagings | writes, cleans up |
| `fulfilment_customers` | Fulfilment customers and their report | reads |
| `webhooks` | Subscribing to an event, checking the signature, reading a payload | writes |
| `shipping_provider_endpoint` | The protocol of your own shipping provider | runs on its own |
| `blocking` | The blocking client, with `--features blocking` | reads |

Picqer cannot delete products, suppliers, purchase orders, packagings, hooks, comments or fulfilment
customers, so the examples that make them reuse what they made before or deactivate it. The
[README](../README.md) says what is left behind.
