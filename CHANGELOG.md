# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-xx-xx
### Added
- [[#27](https://github.com/andrei-ng/u-blox.rs/pull/27)] Merge [ublox-rs/ublox/pull/24](https://github.com/ublox-rs/ublox/pull/24) PR that adds `UBX-NAV-RELPOSNED` into this repository.
    - remove duplicate package definition `MgaGpsEph`
    - add features flags to differentiate between uBlox Series 8 and uBlox Series 9 devices; As  `UBX-NAV-RELPOSNED` has different lengths depending on the protocol /series.