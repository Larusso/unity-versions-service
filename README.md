unity-versions-service
=====================

A simple server application which provides static endpoints for released unity versions.
There are two usable endpoints:

* `/versions` - will return a `json` object with all available versions and release hashes
* `/versions/${VERSION}/hash` - returns the release hash for a given version

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)
[![Build Status](https://travis-ci.org/Larusso/unity-versions-service.svg?branch=master)](https://travis-ci.org/Larusso/unity-versions-service)

Build with cargo
----------------

_just building the binary_

`cargo build --release`

_running it locally with cargo_

`cargo run --bin unity-version-service`

Routes generation
-----------------

All [iron] routes are statically generated at compile time. Cargo will run `build.rs` at compile time which will generate a helper function `pub fn _add_version_routes(router:&mut Router)` and a static `json` string for he `/versions` endpoint from the [versions.yml].

Usage
-----

This package contains two tools
* __unity-versions-service__ - a rust iron server application
* __update_versions__ - a helper tool to fetch latest version of unity and push changes to remote github repository

_unity-version-service:_
```
unity-versions-service - A simple webserver to deliver unity version information

Usage:
  unity-versions-service [options]
  unity-versions-service (-h | --help)

Options:
  --port=PORT       the server port number
  -v, --verbose     print more output
  -d, --debug       print debug output
  --color WHEN      Coloring: auto, always, never [default: auto]
  -h, --help        show this help message and exit
```

_update_versions:_
```
update-versions - Fetch latest versions and update versions.yml on repo.

Usage:
  update-versions [options]
  update-versions (-h | --help)

Options:
  --token=TOKEN         a github auth token
  --message=MESSAGE     the commit message to use
  --repo-name=REPO      name of the repo
  --repo-owner=OWNER    owner of the github repo
  -f, --force           force refresh of the list
  -v, --verbose         print more output
  -d, --debug           print debug output
  --color WHEN          Coloring: auto, always, never [default: auto]
  -h, --help            show this help message and exit
```

Available versions
-----------------

At the moment most released versions of `2017` and `2018` are available (see [versions.yml]). New versions are automatically added through the `update_versions` tool running on Heroku. Missing versions can easily be added by editing the [versions.yml] file in the root of this repository.

License
-------
[Apache License 2.0](LICENSE)

[versions.yml]: versions.yml
[iron]:         https://github.com/iron/iron
