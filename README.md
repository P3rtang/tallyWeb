# TallyWeb
[![Docker Image CI](https://github.com/P3rtang/tallyWeb/actions/workflows/docker-image.yml/badge.svg)](https://github.com/P3rtang/tallyWeb/actions/workflows/docker-image.yml)

## About
### website
visit [tallyWeb](https://tally-web.com) for a public version of the site.
It should work for both desktop browsers and phones.

### Shiny Pokemon
Meant to keep track of hunting shiny pokemon,
but can be used for anything that a counter with timer is useful for.

### leptos
TallyWeb is a website build using the leptos.rs framework

### build information
For now it's not possible to run your own locally hosted version of the site with docker compose, though this is almost ready,
The only thing holding me back is settings up the database which will be looked into...

To run and build yourself clone the git repo and run
```
make dev
```
Prerequisites are running on linux and installing
- docker
- make
- cargo
This should set up the database for you and then run
```
cargo leptos serve --release
```

### requests
if you have a request for a feature open a github issue

## Roadmap
### V0.1
- [x] basic functionality
- [x] backend stable

### V0.2.*
- [x] more infobox information
- [x] sort and search the treeview

### V0.3
- [ ] progress calculation for more hunting methods (e.g DexNav, Raids...)
- [ ] counters can hold subcounters (not just phases)
- [ ] custom infobox layout with reorderable widgets

### V0.4
- [ ] your own server with a docker image
- [ ] without account support
- [x] offline support
- [ ] ...
