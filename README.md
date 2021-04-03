## Portal - Full Text Search Web Service

[![crates.io](https://meritbadge.herokuapp.com/portal)](https://crates.io/crates/portal)

### Purpose

The purpose of this service is to be your full text search web service.

This service is a combination of [tide](https://crates.io/crates/tide), [broker](https://crates.io/crates/broker), and [sonic-server](https://crates.io/crates/sonic-server)

It uses tide to provide a web server that JSON clients can authenicate (using broker) and then index, search, and suggest (using Sonic). 

Currently this is in development and a stub project.
