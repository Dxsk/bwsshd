# Changelog

## [1.1.0](https://github.com/Dxsk/bwsshd/compare/v1.0.1...v1.1.0) (2026-11-24)


### Features

* add bwsshd install/uninstall and a Windows MSI ([d2418b2](https://github.com/Dxsk/bwsshd/commit/d2418b268047deee46b6a498d05a6569a69e99f0))
* replace the Windows MSI with an NSIS setup ([f88f455](https://github.com/Dxsk/bwsshd/commit/f88f45567af9fab024258e9fe97a7684ad298d87))
* **tray:** link to the GitHub project from the menu and the panel ([93c6e20](https://github.com/Dxsk/bwsshd/commit/93c6e2067b910fcf702febce59cdef342f73b7e1))


### Bug Fixes

* **msi:** give the main component a fixed GUID ([025e2d4](https://github.com/Dxsk/bwsshd/commit/025e2d431f2842f576054b36719f2ca71842d0ad))
* **tray:** open links again on Windows after freeing the console ([01f5f55](https://github.com/Dxsk/bwsshd/commit/01f5f551ea4fda14c887f8f048c67936c0409f5d))

## [1.0.1](https://github.com/Dxsk/bwsshd/compare/v1.0.0...v1.0.1) (2026-11-24)


### Bug Fixes

* **release:** attach binaries before publishing the release ([8eb6caa](https://github.com/Dxsk/bwsshd/commit/8eb6caad924cd5a96102ea70b5819dab55317ba4))

## [1.0.0](https://github.com/Dxsk/bwsshd/compare/v0.2.0...v1.0.0) (2026-11-24)


### Features

* let a key name pick its own Host with (alias,address) ([d2c0e60](https://github.com/Dxsk/bwsshd/commit/d2c0e605ca208d9bb8319c060a330e2909bf3a75))
* rewrite bwsshd in Rust ([57a171b](https://github.com/Dxsk/bwsshd/commit/57a171bc179c0d3884132c9420e94ce9fa040b99))
* **site:** add an IPv6 example to the naming playground ([1f3b846](https://github.com/Dxsk/bwsshd/commit/1f3b8465fe210c5b762227cd4afff7a8f8d0721d))
* **site:** add who it's for and Bitwarden agent setup ([2a7d2ae](https://github.com/Dxsk/bwsshd/commit/2a7d2aeedbaffdd516cb8f13397fa3968e20e69c))
* **site:** cover multi-device setups and link who it's for ([136815f](https://github.com/Dxsk/bwsshd/commit/136815fdf8fecc593445eb900f6de46394a6a24f))
* tray icon with last sync, resync and quit ([618bd4a](https://github.com/Dxsk/bwsshd/commit/618bd4acc00d7ebe608de83a27d3e2956e1a3cef))
* tray panel on left click and open the generated config ([b5db99b](https://github.com/Dxsk/bwsshd/commit/b5db99b83cb630f5d13947947490a2e897b53516))
* **tray:** pause auto sync and pick the sync interval ([c2df757](https://github.com/Dxsk/bwsshd/commit/c2df757f0435a7e03c5ef0ad87db7a38183d1938))
* **tray:** remember the pause and the sync interval ([73c65b1](https://github.com/Dxsk/bwsshd/commit/73c65b17501ba6d04944f01bcd2d2f9d4cb355bf))
* **tray:** tell when a newer release is out ([4d1b59e](https://github.com/Dxsk/bwsshd/commit/4d1b59e4e26f902bb3645b3f8e9703fabad579cf))


### Bug Fixes

* keep the tray panel usable after opening the config ([a9e6842](https://github.com/Dxsk/bwsshd/commit/a9e684200935226e6736d564738536c643199ae6))
* **macos:** drop a mut only Windows needs ([61d5c57](https://github.com/Dxsk/bwsshd/commit/61d5c57aa19aedfee04b8fbf4320dc631c87a4bc))
* **windows:** retry when the agent pipe is busy ([9397df7](https://github.com/Dxsk/bwsshd/commit/9397df7f28a72a3e9e61a714f0234e547eb9d8c3))
* **windows:** stop the running daemon before upgrading ([8c334f8](https://github.com/Dxsk/bwsshd/commit/8c334f8e40b4f20a19d3b4c001361e0a62abf7ac))


### Performance Improvements

* **tray:** create the panel webview only while it is open ([0abcaf5](https://github.com/Dxsk/bwsshd/commit/0abcaf5a214b6ce9e5ad81604e25ef05c37084ca))

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
