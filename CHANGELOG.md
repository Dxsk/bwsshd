# Changelog

## [0.3.0](https://github.com/Dxsk/bwsshd/compare/v0.2.0...v0.3.0) (2026-11-24)


### Features

* let a key name pick its own Host with (alias,address) ([fcf67e1](https://github.com/Dxsk/bwsshd/commit/fcf67e16c2ac327950fab5226b34c762de49f5b1))
* rewrite bwsshd in Rust ([5fcc38a](https://github.com/Dxsk/bwsshd/commit/5fcc38a61c525f4d1f1313af2d4640ef62f65ba2))
* **site:** add an IPv6 example to the naming playground ([735af8f](https://github.com/Dxsk/bwsshd/commit/735af8f91cc934d2211bafd7bb2d532aa30896c9))
* **site:** add who it's for and Bitwarden agent setup ([13eaddc](https://github.com/Dxsk/bwsshd/commit/13eaddcb25630158303176752abb125674fdf10e))
* **site:** cover multi-device setups and link who it's for ([87533da](https://github.com/Dxsk/bwsshd/commit/87533dade2e791eb7aa520dded9dce5b0406b5eb))
* tray icon with last sync, resync and quit ([c4738dd](https://github.com/Dxsk/bwsshd/commit/c4738dda9591c6c1743aeee4c072bd25b0af990d))
* tray panel on left click and open the generated config ([0a12b97](https://github.com/Dxsk/bwsshd/commit/0a12b9772222f1b2faf528c2b62c7798ac68ca95))
* **tray:** pause auto sync and pick the sync interval ([b87d033](https://github.com/Dxsk/bwsshd/commit/b87d0337fd2d75e141254453c9a34c16f89bba67))
* **tray:** remember the pause and the sync interval ([a6b2c2f](https://github.com/Dxsk/bwsshd/commit/a6b2c2f95c9012418fdf801a5ea5f87b07562644))
* **tray:** tell when a newer release is out ([a04d16b](https://github.com/Dxsk/bwsshd/commit/a04d16b034e16a067f582e4e57f8757add7057a4))


### Bug Fixes

* keep the tray panel usable after opening the config ([0e1fb41](https://github.com/Dxsk/bwsshd/commit/0e1fb41b98a942319fd8943092a649c2232f1fcc))
* **macos:** drop a mut only Windows needs ([fedd8d1](https://github.com/Dxsk/bwsshd/commit/fedd8d1e5a2905abf429b5dccbc9fa44917856d7))
* **windows:** retry when the agent pipe is busy ([5921fcb](https://github.com/Dxsk/bwsshd/commit/5921fcb47da52f456d95fe53ac16d575822067ea))
* **windows:** stop the running daemon before upgrading ([6d5eb07](https://github.com/Dxsk/bwsshd/commit/6d5eb07bd6f10921b02fe18e6d660a897678c035))


### Performance Improvements

* **tray:** create the panel webview only while it is open ([97d8ba4](https://github.com/Dxsk/bwsshd/commit/97d8ba4ee6a67229484ed9ba2b48d418cf71e167))

## [0.2.0](https://github.com/Dxsk/bwsshd/compare/v0.1.0...v0.2.0) (2026-07-24)


### Features

* add Windows support ([f23d8cc](https://github.com/Dxsk/bwsshd/commit/f23d8ccc09eb6a92bbd65af3fdc0f84be08bdbb4))
* add Windows support ([bb1e997](https://github.com/Dxsk/bwsshd/commit/bb1e997dec41090a8b5963302a921da5509a81a6))

## 0.1.0 (2026-06-27)


### Features

* add -version flag ([02524c4](https://github.com/Dxsk/bwsshd/commit/02524c41d2e6c13947cdf2cbd9e7ebda3c6f5a77))


### Bug Fixes

* address open issues [#1](https://github.com/Dxsk/bwsshd/issues/1)-[#5](https://github.com/Dxsk/bwsshd/issues/5) ([37e431a](https://github.com/Dxsk/bwsshd/commit/37e431adfe3b71b2768373f6324b9a6a08455a7d))
* repair release pipeline and reset for a clean v0.1.0 ([820ca9a](https://github.com/Dxsk/bwsshd/commit/820ca9a123e4673a08306cb646f18bb7a1a57709))


### Miscellaneous Chores

* release 0.1.0 ([300d522](https://github.com/Dxsk/bwsshd/commit/300d5223735794a6068c22a1f2d8af30a4d5a2db))
