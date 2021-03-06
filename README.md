## Portal - Full Text Search Web Service

[![crates.io](https://meritbadge.herokuapp.com/portal)](https://crates.io/crates/portal)

### Purpose

The purpose of this service is to be your full text search web service for any JSON client including JavaScript front-ends like React with fetch.

Portal is a full replacement for [ElasticSearch](https://www.elastic.co/) and [MeiliSearch](https://github.com/meilisearch/MeiliSearch). These services are too complex, too resource hungry, and too slow.

Portal is built to be simple and blazing fast with JWT verification, indexing, deindexing, search, and suggest. 

Indexing is batch indexing by default.

To use this service you need to have a running [sonic](https://crates.io/crates/sonic-server) server and a [broker](https://crates.io/crates/broker) server.

### Features

* Very performant with almost no CPU and memory usage
* Supports 87 natural languages
* Under 500 lines of code
* Supports CORS
* Supports JWT authenication
* Multi-tenant
* Supports SSL - full end-to-end encryption
* JSON API
* Auto-provision and renews SSL cert via LetsEncrypt
* Built on [Sonic](https://crates.io/crates/sonic-server) and [Broker](https://crates.io/crates/broker)

### Use

- create a user on [broker](https://crates.io/crates/broker), login, and get a JWT - then attach the JWT as an Authorization: Bearer {token} to the following JSON API endpoints

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

- Supported Locales:
    * afr
    * aka
    * amh
    * ara
    * azj
    * bel
    * ben
    * bho
    * bul
    * cat
    * ceb
    * ces
    * cmn
    * dan
    * deu
    * ell
    * eng
    * epo
    * est
    * fin
    * fra
    * guj
    * hat
    * hau
    * heb
    * hin
    * hrv
    * hun
    * ibo
    * ilo
    * ind
    * ita
    * jav
    * jpn
    * kan
    * kat
    * khm
    * kin
    * kor
    * kur
    * lat
    * lav
    * lit
    * mai
    * mal
    * mar
    * mkd
    * mlg
    * mya
    * nep
    * nld
    * nno
    * nob
    * nya
    * ori
    * orm
    * pan
    * pes
    * pol
    * por
    * ron
    * run
    * rus
    * sin
    * skr
    * slk
    * slv
    * sna
    * som
    * spa
    * srp
    * swe
    * tam
    * tel
    * tgl
    * tha
    * tir
    * tuk
    * tur
    * uig
    * ukr
    * urd
    * uzb
    * vie
    * ydd
    * yor
    * zul

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
- the secure flag (https) and can be true or false - default `false`
- the certs flag is the storage path of LetsEncrypt certs - default `certs`
- the db flag is the path where the embedded database will be saved - default `db`
- the domain flag is the domain name (e.g. api.broker.com) of the domain you want to register with LetsEncrypt - must be fully resolvable 
- the sonic_server flag is the sonic domain/ip/port of the sonic server - default `localhost:1491`
- the sonic_password flag is the sonic password set in the sonic config file - default `SecretPassword`
- the broker flag is the broker domain/ip/port of the broker server - default `http://localhost:8080`
- production example: `./portal --secure="true" --domain="index.broker.com" --sonic_server="sonic.broker.com" --sonic_password="wj34T%$Dx" --broker="https://broker.broker.com"`

### Service

There is an example `systemctl` service for Ubuntu called `portal.service` in the code

### TechStack

* [Tide](https://crates.io/crates/tide)
* [RocksDB](https://crates.io/crates/rocksdb)

### Inspiration

* [Broker](https://crates.io/crates/broker)
* [Sonic](https://crates.io/crates/sonic-server)
* [ElasticSearch](https://www.elastic.co/)
* [MeiliSearch](https://github.com/meilisearch/MeiliSearch)
