# Changelog

## 0.1.0 - 2026-11-09

Initial release.

- Async client with authentication, validated request types, errors with the code of the API, rate limit handling (opt-in retry and throttle) and webhook signature verification.
- A blocking client behind the `blocking` feature.
- Examples in `examples`, one for every part of the crate, with an overview in `examples/README.md`.
- Products: products, product fields, tags, pricelists, stock, the stock history, parts and compositions, images, warehouse settings, price histories, expected deliveries, the locations of a product and moving stock.
- Orders and customers: orders, order fields, customers, customer fields, comments and backorders.
- Warehouse: warehouses, VAT groups, locations, location types and their stock history, picklists, picklist batches and their presets, picking containers, packagings, packing stations, shipping providers, shipments, templates and users.
- Purchasing and returns: suppliers, purchase orders, receipts and returns.
- Other: tasks, webhook subscriptions, stats, webshop orders, fulfilment customers and their reports, and the protocol of custom and external shipping providers.

A snapshot of the Picqer documentation is kept in `tests/docs` and compared with the published documentation every week.
