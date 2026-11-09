# picqer-rs

Async Rust client for the [Picqer](https://picqer.com) API.

> This is an unofficial, community-maintained library. It is not affiliated with, endorsed by or supported by Picqer.

```toml
[dependencies]
picqer-rs = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The full API documentation is on [picqer.com](https://picqer.com/en/api).

## Usage

```rust
use picqer_rs::orders::{OrderQuery, OrderStatus};
use picqer_rs::pagination::Page;
use picqer_rs::{Client, UserAgent};

#[tokio::main]
async fn main() -> picqer_rs::Result<()> {
    let client = Client::from_env(UserAgent::new("My integration", "dev@example.com")?)?;

    let query = OrderQuery::new().status(OrderStatus::Processing);
    for order in client.orders().list(&query, Page::FIRST).await? {
        println!("{} {}", order.orderid, order.deliveryname.as_deref().unwrap_or("-"));
    }
    Ok(())
}
```

`Client::from_env` reads `PICQER_SUBDOMAIN` and `PICQER_API_KEY`, see [Configuration](#configuration). Create an API key in your Picqer account under Settings > API Keys. To pass the subdomain and the key yourself, use `Client::new(Subdomain::new("example")?, ApiKey::new(key)?, user_agent)?`.

Making an order works the same way. The amount of a line is a `NonZeroU32` and an order needs at least one line, so what Picqer would silently accept, such as an order without lines or a negative amount, does not compile or is refused before it is sent:

```rust
use std::num::NonZeroU32;

use picqer_rs::orders::{NewOrder, NewOrderProduct};
use picqer_rs::types::{CustomerId, NonEmpty, ProductId};
use picqer_rs::Client;

async fn make_order(client: &Client) -> picqer_rs::Result<()> {
    let line = NewOrderProduct::by_id(ProductId(1234), NonZeroU32::new(2).unwrap());
    let order = NewOrder::for_customer(CustomerId(5678), NonEmpty::new(vec![line])?)
        .reference("webshop-1001");

    let created = client.orders().create(&order).await?;
    println!("{} is {:?}", created.orderid, created.status);

    client.orders().process(created.idorder).await?;
    let order = client.orders().get(created.idorder).await?;
    println!("{} is now {:?}", order.orderid, order.status);
    Ok(())
}
```

Picqer makes the order a concept. `process` turns it into picklists in the background, so the order that comes back from `get` can still show its old status for a moment, see [Orders](#orders).

### Configuration

| Variable | What it is | Example |
|---|---|---|
| `PICQER_SUBDOMAIN` | The subdomain of your Picqer account: the `example` in `https://example.picqer.com` | `example` |
| `PICQER_API_KEY` | An API key of that account | |

`Client::from_env(user_agent)` reads both. Use `Subdomain::from_env()` and `ApiKey::from_env()` to read them one at a time, or build the client with `Client::new` and read your configuration however you like. The user agent is not read from the environment: it names your application and a way to contact you, so it belongs in your code.

The live tests read the same two variables.

Picqer asks every integration to identify itself with the name of the application and a website or an email address, so a `UserAgent` is required. The crate adds its own name and version.

### Types instead of strings

What you send is validated when you construct it: a `Subdomain` is a DNS label, an `ApiKey` has no whitespace or `:`, a `UserAgent` carries a way to contact you. Nothing is truncated, dropped or replaced by a default on the way: a value that does not fit is an error.

What the API reports is read more leniently. Picqer extends its enums, so a value this crate does not know is returned as `Reported::Unknown` with the raw value instead of failing the whole response.

### Errors

Failed requests return `Error::Api` with the status, the error code and the message of the API. The code is an `ErrorCode`, or `Reported::Unknown` for a code this crate does not know:

```rust
match result {
    Err(e) if e.is_not_found() => println!("no such object"),
    Err(e) => return Err(e),
    Ok(value) => {}
}
```

Picqer writes its messages in the language of the account, so match on the code, not on the message. A response that is not JSON, such as the page for refused credentials, is kept as the start of the body, cut to 500 characters. It can contain personal data, so think twice before logging it.

### Rate limit

Picqer allows 500 requests per minute per API key, counted again from every full minute. `client.rate_limit()` returns what the last response reported: the limit, what is left and how long until the count starts again.

When the limit is reached the request fails with `Error::RateLimited`. Picqer ignores such a request, so it is safe to send it again. Two opt-in settings help:

- `Client::builder(..).retry_on_rate_limit(retries)` waits until the next full minute and sends the request again, at most `retries` times;
- `Client::builder(..).throttle()` waits before sending a request when the last response reported that nothing was left.

Requests in flight at the same time are not counted by `throttle`, so it lowers the number of `429` responses but does not rule them out.

### Webhooks

```rust
let secret = WebhookSecret::new(secret)?;
verify_signature(&secret, raw_body, signature_header)?;
```

`verify_signature` checks the `X-Picqer-Signature` header: the base64 encoded HMAC-SHA256 of the raw body with your secret as key. Pass the body exactly as received, before parsing it.

### Timeouts, redirects and TLS

Requests time out after 60 seconds (10 seconds to connect); change it with `Client::builder(..).timeout(..)`. The default HTTP client does not follow redirects: the API is not expected to send any, and following one could carry your key to another host. If you pass your own `reqwest::Client`, its settings apply as is.

The base url is `https://{subdomain}.picqer.com/api/v1/` and must use `https`. Plain `http` is only accepted for loopback addresses, which is useful for mock servers. The API key is never shown by `Debug` and sent as a sensitive header. The crate's own copy is wiped from memory when dropped; the copy in a request header, held by the HTTP client while the request runs, is not.

Nothing is retried unless you ask for it.

### Blocking client

With the `blocking` feature, `picqer_rs::blocking::Client` has the same operations without `async`. Build it with `Client::builder(..).build_blocking()` or `blocking::Client::new(..)`. Do not use it from inside an async runtime. A client passed to `http_client` is ignored by the blocking client.

## Features

| Feature | Default | |
|---|---|---|
| `rustls-tls` | yes | TLS through rustls |
| `native-tls` | no | TLS through the platform library |
| `blocking` | no | The blocking client |

Use `default-features = false, features = ["native-tls"]` to switch TLS backends. The minimum supported Rust version is 1.88.

## Covered

Packagings, shipping providers, shipments, receipts, returns, backorders, tasks, picking containers, webhook subscriptions, stats, webshop orders, picklist batches and their presets, locations, the locations of a product, moving stock, location types and the stock history of locations, templates, users, packing stations, purchase orders, suppliers, picklists, orders, comments, customers (with their addresses and tags), customer fields, order fields, warehouses (with their stock), VAT groups, tags, pricelists, product fields, products (including their tags, pricelist prices, stock, parts, compositions, images, warehouse settings, price histories and expected deliveries) and the stock history. Every list has a call for one page and a call that follows all pages. The sections of the [API documentation](https://picqer.com/en/api) that are not listed here are added step by step.

### Money, dates and units

An amount is a `Money`, kept in cents. Picqer stores two decimals and accepts negative amounts, so `Money` does too. Dates are `DateTime`s without a time zone, in the local time of the account, as Picqer reports them. A weight is `Grams` and a length `Centimeters`: building one rejects a negative value, but reading one does not, because the API stores what it is given.

### Pages

Picqer returns at most 100 results and pages by `offset`. Some endpoints, such as the one for products, round an offset down to a multiple of 100, and others skip exactly that many results. A `Page` counts whole pages from zero, which works the same on all of them.

### Stock

`products().update_stock` adds or removes stock with `StockUpdate::adjust`, or sets it with `StockUpdate::set`. A change needs a reason and cannot be zero, and the target is always named: a location or `StockTarget::NoSpecificLocation`. Picqer is more lenient: it replaces an empty reason with "No reason given", truncates `1.5` to `1`, and lets stock go below zero although its documentation says it cannot, so what Picqer reports can be negative.

Setting an amount does not take a location, as in the API.

### Orders

`orders()` lists, fetches and finds orders, creates and changes them, changes their lines and tags, and moves them through their statuses: `process`, `pause`, `resume`, `change_to_concept`, `reopen`, `cancel`, `force_cancel` and `undo_cancellation`, plus `allocate`, `deallocate`, `prioritise`, `process_backorders` and `anonymize`.

An order is made for a customer or for a guest. A guest order needs both addresses, which each need a name and a country. At least one line is required and its amount is positive. Picqer accepts an order without lines, a zero or negative amount, and an invalid email address, country or discount, and silently truncates `1.5` to `1`. This crate is stricter, as it is elsewhere. A line can name its product by id or by code when an order is made, but only by id when it is added later. Adding a product the order already has adds to its line.

Picqer does the work of a status change in the background. The order that comes back still has its old status, so fetch it again a little later. While Picqer is busy with an earlier request for the order, such as allocating stock, it refuses another with `ErrorCode::OrderBeingProcessed`. Cancelling a processed order needs `force_cancel`, which takes its picklists away too. A cancelled order stays in Picqer, and `undo_cancellation` makes it a concept again. A change to an order ignores the fields it may not change, such as the status.

The status of an order, its `orderid` and the tags are read as described for reading orders. The orders of a customer made through a fulfilment customer can only hold products of that fulfilment customer. The deprecated notes are not covered.

Order fields and customer fields are made in the Picqer app and all six types were checked. Order fields are read with `order_fields()`. `NewOrder::orderfields` sets values when an order is made, and `orders().set_field`, `fields` and `remove_field` change them later, one field per call. Picqer reports a value as text, also when a number was sent, and does not check a value against the values of a radio, select, checkbox or multicheckbox field. `OrderField::value` checks that a radio, select or checkbox value is one of the listed values, because the app lets a checkbox take only one, and `choices` writes the ticked values of a multicheckbox with a semicolon between them, as the app does. An empty text removes the value of an order field, but a customer field keeps it as an empty value, so remove that one with `cleared`. Customer fields work the same way: `CustomerField::value`, `choices` and `cleared` build what `NewCustomer::customerfields` and `CustomerChanges::customerfields` take, and a single customer reports the fields that have a value. Picqer does not enforce a required order or customer field when an order or customer is made through the API.

Picqer's documentation calls the number of an order (`orderid`) an integer, but it is text such as `O2026-1599`. A status Picqer reports that this crate does not know is kept as `Reported::Unknown`. The tags of an order are an object when it has some and an empty list when it has none, and both are read. The picklists of an order, `pickup_point_data` and `comment_count` are not read yet.

### Suppliers

`suppliers()` lists, fetches, creates and changes suppliers. Picqer cannot delete a supplier and requires its name to be unique. Picqer replaces a language it does not offer with `en` and does not check the country, so only `nl` and `en` and an ISO country can be sent. `active`, `idtemplate`, `minimum_order_total`, `full_address` and `comment_count` are reported but not documented, and are not read.

### Packagings, shipping providers, templates and users

`packagings()` lists (active or inactive), fetches, creates and changes packagings; Picqer cannot delete one, so deactivate it with `PackagingChanges::active(false)`. Picqer does not require a name or barcode to be unique, refuses a dimension that is negative or a fraction, and does not document the unit of the dimensions. A validation error on a packaging comes back as `422` with only an `errors` object, and its text is shown in the error. `shipping_providers()` lists the providers with their profiles. `templates()` and `users()` read templates and users; `users().me()` is the user the API key belongs to, and only a single user comes with its rights.

### Locations, location types and their stock history

`locations()` lists, fetches, makes, changes and deletes locations and containers, lists the products linked to a location and changes the destination Picqer suggests for a product in a container. `location_types()` does the same for location types and makes one the default, and `location_stock_history()` reads the stock changes per location, including movements between locations. Both kinds of location are one type: `NewLocation` makes a location, `NewContainer` a container, and `LocationQuery::kind` lists containers, because Picqer lists only locations by default.

Picqer requires the name of a location and of a location type to be unique, and answers `400` with code 5 otherwise. The name filter finds exact names only. It does not check the color of a location type, so `red` and `#GGGGGG` are accepted and the crate sends only `#` and six hexadecimal digits, but it reads any color so that one such type cannot make a list fail. A container is named `Container` and the next number: Picqer ignores a name that is given, and ignores a new name when a container is changed. The `remarks` of a container can be changed, and a container has no other fields to read. It ignores a `type` it does not know and makes a location, which cannot be sent here. A new location answers without its flags, and they are `false`. `DELETE` answers `200` with `null` instead of `204`. A location is taken out of its parent with `null`, and Picqer refuses a parent that would make a circle.

`auto_link_to_parent` looks for a parent in the name and makes the parents that are missing: `ZZ.1.2` made `ZZ.1` and `ZZ`, and `ZZQ9.8.7` made `ZZQ9.8`, `ZZQ9` and `ZZQ`, but `ZZPR.1.2` made none, and the rule is not documented. Only the filters that the documentation lists exist, and `only_available` counts by being present: any value means yes. `idlocation_type` filters on the type a location has, which on a developer account can be a type the type list does not hold: its sample locations have type 1, and the list only the default type 9380.

### Webhooks

`hooks()` manages the subscriptions to the events of Picqer, which it calls hooks: list, fetch, create, deactivate and reactivate. `WebhookEvent` lists the 49 events of the documentation. A hook has an event and an address, a name of your own and, optionally, a secret that Picqer uses to sign every delivery, which `verify_signature` checks; Picqer only says whether a hook has a secret. `WebhookPayload` reads the body of a delivery; its `data` depends on the event, so it is kept as sent and read with `data_as`. A developer account cannot receive deliveries, so the payload follows the documentation.

Picqer cannot delete a hook: `DELETE` deactivates it, and the hook stays in the list, also when it is inactive. A hook cannot be changed either, and Picqer refuses a second hook for the same event and the same address with code 5, whatever its name. Reactivating a hook that is active gives code 21, and deactivating one twice changes nothing. Picqer accepts an `http` and an `https` address and refuses other forms and events it does not know, which `NewHook` and `WebhookEvent` do too. Picqer deactivates a hook that keeps failing, after five complete failures within 24 hours. The live tests use hooks of their own, made once for an event and an address nobody triggers, and leave them deactivated.

### Picking containers

`picking_containers()` lists (filtered by picklist), fetches, makes, renames and deletes picking containers, and links a picklist to one. Picqer requires the name to be unique and not empty; it takes a long name, and answers a name that is a number as a number when the container is made and as text after that, so the name is read as text. A container holds one picklist: linking another one takes the place of the first, and linking a picklist that is in another container takes it out of that one. Picqer has no call to unlink a picklist: deleting the container does it, also when a picklist is linked, and it answers code 7 for a picklist it does not know. Whoever changes a container has to send its name, since Picqer refuses a change without one, so `rename` is the only change. The `picking_container` that picklists and batches show was `null` also when a container was linked, so it is not read.

### Webshop orders

`webshop_orders()` lists (filtered with `WebshopOrderQuery`) and fetches the orders of webshop integrations, with how far their import is. They follow the documentation: a developer account has no webshop integration, so the list is empty and no response has been compared with the types. The tests use the examples of the documentation. The status values of the documentation are given as examples, so another status is kept as `Reported::Unknown`, and the identifier and number from the webshop are read as text also when Picqer sends a number.

### Stats

`stats()` lists the stats that exist with what they tell, and fetches the value of one. `StatKey` has the twelve keys of the documentation, all of which a developer account answered with a whole number, so a value is read as one; `get_by_key` takes a key it does not have, and a key the crate does not know comes back from the list as `Reported::Unknown`. Picqer answers a key that does not exist with not found.

### Tasks

`tasks()` lists (filtered with `TaskQuery::completed`), fetches, creates, changes and deletes tasks, and completes and uncompletes them. The errors of tasks come back as `{"errors": …}` without a code or a message, which `ApiError::errors` holds and the text of the error shows.

Picqer takes the assignee away from a task whenever a change does not name one, even a change of only the title, and an empty change too. So `tasks().update` fetches the task first and sends its assignee along when `TaskChanges` does not set one, which costs a call; `assigned_to` and `unassigned` name it. The other fields stay when they are not sent. A deadline date cannot be taken away: Picqer makes it today when it is emptied, so only the deadline time can be removed, with `without_deadline_time`. The deadline time must be `HH:MM:SS`: Picqer refuses `HH:MM` with a message that asks for `HH:MM`. A deadline date is read as a date and time at midnight. Picqer takes an empty description as none and a long title as it is, but refuses an empty title, which `TaskTitle` does too. It parses a date it is given in another form instead of refusing it, so only `YYYY-MM-DD` is sent. Completing a task that is completed changes nothing, and any value for `completed` in a list is read as true except `false`. `idrecurring_task`, which Picqer reports and does not document, is not read.

### Backorders

`backorders()` lists backorders (filtered with `BackorderQuery`), fetches one, lists those of an order, deletes one and makes picklists of the backorders that have stock available. Deleting a backorder cancels its line of the order, and an order whose lines are all cancelled is cancelled too; `undo_cancellation` brings the line back. `process` answers an empty list, also when it made picklists, so the answer is not read: look at the picklists or the backorders afterwards. `BackorderQuery::since_id` also lists the backorder with that id, although the documentation says a later one. A backorder becomes available when stock arrives in its warehouse, which can be another warehouse than the first. The documentation calls `date_available` a date and time and shows a date, and a developer account answered `null` for it and for `is_purchased` (`false`) in every case, also with a purchased purchase order for the product, so `date_available` is kept as the text Picqer sends. `parts_per_parent`, which the documentation shows with `null`, is not read, because its type is not known.

### Returns

`returns()` lists, fetches, creates, changes and deletes returns, handles their returned products and replacement products, reads and adds to their logbook, and registers what happened to returned products with `receive`. `return_statuses()` and `return_reasons()` list the statuses and the reasons, which a return and its products refer to by id. A single return comes with its status, customer, order, products and totals; a list has only the return itself.

Picqer needs only a status to make a return: it takes the default template when none is given, although its documentation says one is required. It does not check the country, the email address or the language, so `ZZ`, `nope` and `xx` are stored, which `NewReturn` refuses. Picqer takes an amount of zero, a fraction as its whole part and a price that is missing as zero, so the amount is a positive number. The replacement products given when a return is made are stored, but the answer does not show them yet. Picqer makes the return before it checks the products, so a product with a reason it does not know makes an empty return and fails. The number of a return is text such as `R2026-1024`, although the documentation calls it an integer, and `status=` in a list takes an id, `processing` or `completed`; any other value lists every return.

The status of a return can be changed with `ReturnChanges` and with a logbook entry, and a status that completes a return sets `completed_at` and the user. A logbook entry with neither a comment nor a status is answered as if it was made, and `NewReturnLog` does not allow one. `notify_customer` sends the customer an email when the return has an address. Products that are expected, received or unsalable can be changed into each other with `receive`. Adding a product to the stock needs a warehouse, and Picqer answers `404` without one, so `ReceiveReturn` checks that. It puts the stock at no specific location unless a location is given. It cannot be undone: the line is no longer changeable (code 33), and a return that has one cannot be deleted. For a product that tracks lots, Picqer answers `422` with code 46 and the errors of each line in `context.errors` of the error, which is `ApiError::context`. The live tests do not add products to the stock for that reason.

### Receipts

`receipts()` lists, fetches, creates, completes and deletes receipts, lists the products that are expected on one with the ways to receive them, receives a product and reverts it. A receipt is made for a purchase order or for a supplier (`NewReceipt`); this crate only makes receipts of version 2, in which a product that is received goes into the stock at once, because Picqer deprecates version 1 and makes one when the version is left out. A list can still hold receipts of version 1, which are read with what they have in common.

A product is received with `ReceiveProduct` and one of three strategies from the documentation: on a line of a purchase order, divided over the open lines (`Automatic`), or added to the purchase order or to a new one (`New`). The answer holds a line for every line of a purchase order the amount was divided over. Picqer accepts more than what is expected. With traceability on the account, a product that tracks lot numbers and expiration dates needs both and a location, and Picqer answers code 46 without them, with a lot number that is empty, longer than 255 characters or not text, or with a date that is not `Y-m-d`. A location holds one lot, so another lot at the same location gets code 44.

A product that is received can be reverted, which takes its amount out of the stock again and marks the line as reverted; this also works after the receipt was completed. A completed receipt takes no more products (code 33). Picqer deletes a receipt that has no products or whose products were all reverted, but not a completed receipt that had products received. A receipt of version 2 can be completed without products, and then deleted. The purchase order of a receipt can still be a concept. `completed_by` of a receipt is an object, as in the documentation's example although its table calls it an array, and so is the `location` of a received product. The live tests receive on a purchase order of their own, revert everything and delete their receipt, so they leave nothing behind.

### Locations of a product and moving stock

`products().locations`, `link_location`, `update_location` and `unlink_location` handle the locations a product is linked to, each with the stock of the product there, a note and whether it is the preferred location. `products().move_stock` moves stock with a `StockMove` between locations, containers and no specific location (`StockPlace`), and returns both ends with the stock of the warehouse afterwards.

Linking a location that is linked already changes nothing, an unknown location is `404` and a missing one is `422` with only an `errors` object. A product has one preferred location per warehouse, and making another one preferred takes it away from the first. `null` takes a note away. The stock in the answer to linking is `null` until the product is read again. Picqer refuses to unlink a location that still has stock of the product with code 33, and refuses to delete a location that has a product linked with code 5. Moving more than there is gives code 43, and moving to the same location code 5. Picqer takes an amount of zero as a movement of nothing, moves a negative amount the other way and reads text as a number, so the amount is a positive number here. `move_claims_of` moves the reservations of one picklist along with the stock; its reservation then points at the new location. A product that tracks lot numbers or expiration dates (`ProductChanges::track_lot_numbers`, with traceability on the account) can have them corrected per location with `update_location`. Picqer takes a lot number of 1 to 255 characters and a date as `Y-m-d`, a date in the past too, answers `422` with only an `errors` object otherwise, and leaves a value that is not sent, or is `null`, as it is, so neither can be cleared. Moving stock between warehouses needs a `reason` and Picqer answers code 5 without one; it works from a location to a location or to no specific location in the other warehouse. The destination Picqer suggests for a product in a container is changed with `locations().set_intent`, and `None` takes it away. Picqer only accepts this for a container and for a product that is in it, with code 46 otherwise.

### Picklist batches

`picklist_batches()` lists batches (filtered with `BatchQuery`), fetches one with its products and picklists, creates one, adds a picklist, takes a picklist out, assigns a batch to a user or to nobody, lists the picklists that have a product and downloads the PDF; `picklist_batch_presets()` reads the presets, which are made in the app. A batch is made from specific picklists or from a preset (`NewBatch`); a preset gives the batch its title and its user, and `preferred_number_of_picklists` overrides how many it takes.

A batch cannot be deleted, and it is completed as soon as it has no picklists left, whether the last one is taken out or cancelled, so a batch that was made and emptied stays behind as a completed one. That is why the live tests do not make batches by default: add a picklist to an open batch and take it out again, which leaves the batch as it was, and `PICQER_LIVE_CREATE_BATCHES=1` makes the test that creates them run. Adding a picklist that is already in another batch moves it. The `assigned_to` and `completed_by` of a batch are objects, as in the example of the documentation, although its table calls them arrays, and `picklist_batchid` is a number although the table calls it text, so it is read as text. The `products` filter in the documentation changes nothing in a list: only a single batch comes with its products. The PDF is the document itself and not, as for a picklist, a JSON object; `includePicklists` and `includePackinglists` only count by being present, even with the value `0`. A preset only says how many picklists match it when it is fetched on its own.

### Shipments

`shipments()` lists shipments (filtered with `ShipmentQuery`: a search of the tracking codes and delivery names, and a creation period), fetches one, lists those of a picklist, lists the shipping methods allowed for a picklist and makes a shipment, which is returned. A shipment has parcels and a parcel has documents, such as its label; `carrier_key` is a `CarrierKey` from the documented list.

A shipment needs a shipping profile, and without one Picqer answers `400` with code 5. It asks the provider for a label, so a provider that refuses, an unknown profile and an unknown packaging all come back as code 26 (`ErrorCode::CannotCreateShipment`), and creating a shipment with the PostNL provider of a developer account fails like that because the account has no PostNL key. Picqer truncates a fraction of the weight, takes a negative weight as zero and accepts text, so the weight is `Grams`. It ignores an extra field the profile does not ask for; the `Manual` provider takes `code` and `tracking_url` through `NewShipment::extrafield`, and without them the shipment has no tracking code. The shipping methods of a picklist say which extra fields they ask for and whether the provider supports several parcels, which the documentation does not show.

The `Manual` provider accepts one parcel and refuses two or more with code 26, so a shipment in several parcels follows the documentation. Picqer has no call to cancel a shipment: cancelling the order with `force_cancel` cancels its shipments. A new shipment can take a few seconds to show in the list. The shipment answers only contain what the documentation and its examples show; the other fields Picqer sends, such as the URL of the label in the shipment itself, are not read.

Packing stations follow the documentation: a developer account cannot have one, because Picqer needs PrintNode for that, so they have not been compared with real responses.

### Purchase orders

`purchase_orders()` lists, fetches, creates and changes purchase orders and their lines, and moves them through their statuses: `mark_as_purchased`, `close` and `cancel`. A purchase order is `concept`, then `purchased`, then `received`; closing one that was not purchased also makes it `received`, and only a concept or purchased one can be cancelled. A purchase order is made for a supplier by id or, when Picqer does not know it, by name. A product can be on a purchase order once, and a purchase order can still be changed after it was purchased. Picqer accepts an order without lines and an amount of zero or less, and rounds `1.5` to `2`; the amount is positive here. The products of a fulfilment customer can only be ordered on a purchase order of that fulfilment customer.

### Picklists

`picklists()` lists, fetches and finds picklists, changes them (`urgent`, `invoiced` and the shipping provider profile, the only fields Picqer takes), and covers what happens in the warehouse: `pick` and `pick_all`, `set_pick_locations`, `assign` and `unassign`, `snooze` and `unsnooze`, `pause` and `resume`, `cancel_product`, `split`, `close` and `cancel`, the packaging advice, and the picklist and packing list as PDF bytes. Picklists are made by processing an order, and a picklist of an order is also in `Order::picklists`, without its lines.

A pick or an undo names the product by its code, its line or one location, with a positive amount. Picqer takes a missing amount as one and truncates a fraction, which cannot be sent here. A line has pick locations, and stock that has no specific location has a pick location without a name. `cancel` answers with an empty list instead of a picklist, and cancelling a closed picklist puts its products back in stock.

### Comments

A comment is added to, and listed for, a `CommentTarget`: an order, return, picklist, picklist batch, supplier, purchase order, receipt, product or customer. Its author, mentions and source are enums, because their shape depends on a type that Picqer reports next to them. A kind this crate does not know comes back as `Other` with the raw value. A comment needs text that is not blank, which Picqer also requires. Prefix the full name of a user with `@` to mention them.

Picqer answers adding a comment with `200` instead of `201`, sends `images` and `reactions` that are not read, and answers a filter on an author it does not know with not found. The number of a picklist batch is a number in a comment on one, and is read as text. Comments by fulfilment customers follow the documentation: the account this was checked against has none.

### Customers

Your own number for a customer is `customerid`. Picqer's documentation calls it an integer, but it reports and accepts text such as `ABC-1`, so it is a `CustomerNumber`. A new address needs a country: the documentation calls it optional, but Picqer refuses to change an address without one. An empty address name would be replaced by the name of the customer, so it is refused too. A discount can be 0 to 100 percent when sent, although Picqer stores any number and reports what it stored. Only `nl` and `en` are accepted as a language; Picqer refused the other languages it was tried with.

The addresses of a customer are changed through their own calls, because Picqer ignores them in a change of the customer.

### Compositions

A product of the type `virtual_composition` or `composition_with_stock` has parts. `parts` lists them and `parts_tree` also lists the parts of the virtual compositions among them. `compositions` is the reverse: the compositions a product is part of. `produce` registers the production of a composition with stock of its own, which takes the parts out of stock.

A part has an amount of at least one. Picqer scopes products to their fulfilment customer, so a part must belong to the same customer as its composition, and it answers "not found" otherwise. It also answers "not found" for the parts of a product that is not a composition.

### Images

`NewProductImage::new` takes the bytes of a JPEG, PNG or WebP image, recognises the format and encodes it for the API. Picqer's documentation names JPEG and PNG only, but it accepts WebP.

### Warehouse settings

A product has settings for a warehouse only once they are set. Picqer applies rules that its documentation does not describe: on the account this was checked against, the desired stock level was only kept when the order level was above zero and the maximum was not stored at all.

### Product fields

A value is built from the field: `field.value("x")` checks it against the values of a radio, select or checkbox field, which Picqer does not (it stores any text); the app lets a checkbox take only one value. `field.cleared()` removes the value, which Picqer does when it is sent as `null`. Picqer reports every value as text, also when a number was sent, only reports the fields that have a value, ignores a field it does not know, and does not enforce `required`. The types are `text`, `textarea`, `radio`, `checkbox`, `multicheckbox` and `select`; another type is kept as `Reported::Unknown`. Picqer stores every value as text: an array is stored as its JSON text and `true` as `1`. The app writes the choices of a `multicheckbox` with a semicolon between them, so build that value with `field.choices([...])`, which checks every choice against the field, and read it with `entry.choices()`. 

### Products

`products().list` lists the active products. `ProductStatus::Inactive` lists the inactive ones: Picqer treats the presence of the `inactive` parameter as asking for them, whatever its value, so the active ones are asked for by leaving it out.

Picqer does not delete products. `inactivate` is the closest: a product can only be made inactive when it has no stock, is in no compositions and is on no open picklists, backorders or purchase orders.

### Fulfilment customers

`fulfilment_customers()` lists, fetches, creates and changes the customers of Picqer Fulfilment, and `report` gives the statistics of one over a period (`ReportPeriod`; without one Picqer reports the current month). Picqer has no call to delete a fulfilment customer, so one that was made stays.

Picqer requires a language when creating one, although its documentation says it is optional, so `NewFulfilmentCustomer::new` takes it. It does not check the email address, a username without a password or a login without a username, and it takes an unknown shipping profile as none. The shipping profiles are only in a single customer, not in the list. The password is write-only and wiped on drop.

### Custom and external shipping providers

`shipping_integration` has the types for the protocol Picqer speaks with an endpoint of yours: `ConfigOptions` for the configuration endpoint, `ShipmentRequest::from_slice` for what Picqer sends to make a shipment, and `ShipmentResult` or `ShipmentErrors` as the answer. Answers are built with validating constructors: a list option needs values, a parcel needs its label as a PDF, and file contents are base64 encoded for you. They follow the documentation and its examples; no endpoint of your own is connected to the account this crate was checked against, so none of it was tested against Picqer. The documentation does not give a type for the values of options, nor for `pickup_point_data`, so those stay `serde_json::Value`. It shows the documents of a parcel as a list in one example and as a list of lists in another: answers are written as a list, and both are read.

## What is not included

Not covered: the deprecated order notes and the old JSON push for custom shipping methods, for which Picqer recommends the newer protocol. The requests Picqer makes to endpoints of yours, for webhooks and shipping providers, are described by types and are not requests this crate makes.

Product fields were checked against real fields of all six types. Pricelists are read only here, as documented.

## Testing

```sh
cargo test --all-features
```

The tests run against a mock server, with fixtures taken from real responses. To exercise a real account, use one meant for development:

```sh
PICQER_SUBDOMAIN=... PICQER_API_KEY=... cargo test --test live -- --ignored
```

The live tests also read a `.env` file in the project root: copy `.env.example` to `.env` and fill it in. Variables that are already set in your environment win over the file. The crate itself never reads `.env`.

The product test leaves one inactive product behind.

### Documentation drift

Picqer publishes no specification, so `tests/docs` holds a snapshot of the structure of its documentation: the endpoints and the attribute tables of every page, and the dates of the changelog. `tests/docs/coverage.json` says for every documented endpoint whether this crate covers it, and `tests/docs.rs` fails when a documented endpoint is not accounted for. A weekly workflow fetches the documentation again and fails when it no longer matches.

```sh
cargo xtask docs           # rewrite the snapshots
cargo xtask docs --check   # fail when the documentation changed
```

Endpoints that the documentation only describes in running text, or does not describe at all, are listed under `not_in_snapshot`.

## Supported API versions

| | Version |
|---|---|
| Picqer API | v1, `https://{subdomain}.picqer.com/api/v1/` |
| Documentation snapshot | https://picqer.com/en/api, in `tests/docs`, taken on 2026-11-09 |

Picqer does not publish a machine-readable specification, so the types follow the documentation and the responses of a developer account. Changes to the API are announced in the [changelog](https://picqer.com/en/api/changelog).

## License

MIT

---

This library is developed by [Solvalutions](https://solvalutions.nl) and [Rob Meijerink](https://robmeijerink.nl).

*&copy;&nbsp;2026 &mdash; Built by Rob Meijerink.*
