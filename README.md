## Portal - Full Text Search Web Service

[![crates.io](https://meritbadge.herokuapp.com/portal)](https://crates.io/crates/portal)

### Purpose

The purpose of this service is to be your full text search web service for JavaScript front-ends like React with fetch.

Portal is a full replacement for [ElasticSearch](https://www.elastic.co/) and [MeiliSearch](https://github.com/meilisearch/MeiliSearch). These services are either too complex, too resource hungry, and/or too slow.

Portal is built to be simple and fast as possible with JWT verification, indexing, deindexing, storage, search, and suggest. Indexing is batch indexing by default.

To use this service you need a valid JWT token from a service like [broker](https://crates.io/crates/broker) and a running [sonic](https://crates.io/crates/sonic-server) server.

### Features

* Very performant with almost no CPU and memory usage
* Under 500 lines of code
* Supports CORS
* Supports JWT authenication
* Multi-tenant
* Supports SSL - full end-to-end encryption
* JSON API
* Auto-provision and renews SSL cert via LetsEncrypt
* Built on [Sonic](https://crates.io/crates/sonic-server) 

### Use

#### Index

```html
POST /index
```
- authenticated endpoint (Authorization: Bearer {jwt})
example:
```json
{
    "items": [{
        "collection": "coffee", 
        "bucket": "tenant_1", 
        "id": "49e28aae-88d4-4c19-86d8-51f2c9f11039", 
        "data": {
            "name": "roasted",
            "image": "https://img.com/bucket/123/123.jpg"
        },
        "locale": "eng",
        "indexes": ["name"]
    }]
}
```
- note: `locale` is an optional field of an [ISO 639-3 locale code](https://iso639-3.sil.org/code_tables/639/data) - if not defined locale will be auto-detected

will return: `200` or `500` or `400` or `401`

#### Search

```html
POST /search
```
- authenticated endpoint (Authorization: Bearer {jwt})
```json
{
    "collection": "coffee", 
    "bucket": "tenant_1", 
    "query": "roasted",
    "limit": 10,
    "offset": 10
}
```
- note: limit and offset are optional fields

will return: `200` or `500` or `400` or `401`

200 - will return an array of objects
```json
[
    {
        "collection": "coffee", 
        "bucket": "tenant_1", 
        "id": "49e28aae-88d4-4c19-86d8-51f2c9f11039", 
        "data": {
            "name": "roasted",
            "image": "https://img.com/bucket/123/123.jpg"
        },
        "locale": "eng",
        "indexes": ["name"]
    }
]
```

#### Suggest

```html
POST /suggest
```
- authenticated endpoint (Authorization: Bearer {jwt})
```json
{
    "collection": "coffee", 
    "bucket": "tenant_1", 
    "query": "r",
    "limit": 10
}
```
- note: limit is an optional field

will return: `200` or `500` or `400` or `401`

200 - will return an array of words (strings)
```json
["roasted"]
```

#### Deindex

```html
POST /deindex
```
- authenticated endpoint (Authorization: Bearer {jwt})
```json
{
    "ids": ["49e28aae-88d4-4c19-86d8-51f2c9f11039"]
}
```

will return: `200` or `500` or `400` or `401`

### Install

``` cargo install portal ```

- the origin can be passed in as a flag - default `*`
- the port can be passed in as a flag - default `8888` - can only be set for unsecure connections
- the jwt_secret (for jwts) should be passed in as a flag - default `secret`
- the secure flag (https) and can be true or false - default `false`
- the certs flag is the storage path of LetsEncrypt certs - default `certs`
- the db flag is the path where the embedded database will be saved - default `db`
- the domain flag is the domain name (e.g. api.broker.com) of the domain you want to register with LetsEncrypt - must be fully resolvable 
- the sonic_server flag is the sonic domain/ip/port of the sonic server - default `localhost:1491`
- the sonic_password flag is the sonic password set in the sonic config file - default `SecretPassword`
- production example: `./portal --secure="true" --jwt_secret="xTJEX234$##$" --domain="index.broker.com" --sonic_server="sonic.broker.com" --sonic_password="wj34T%$Dx"`

### TechStack

* [Tide](https://crates.io/crates/tide)
* [RocksDB](https://crates.io/crates/rocksdb)

### Inspiration

* [Broker](https://crates.io/crates/broker)
* [Sonic](https://crates.io/crates/sonic-server)
* [ElasticSearch](https://www.elastic.co/)
* [MeiliSearch](https://github.com/meilisearch/MeiliSearch)
