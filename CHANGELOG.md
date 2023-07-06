# Changelog

## [0.3.6] - 2023-07-07
### Changed
- fix deserialization in the presence of device groups
- detect 403 responses and return FritzError::Forbidden


## [0.3.5] - 2023-02-13
### Added
- add `fritz_api::trigger_high_refresh_rate` to increase the update rate of watts, voltage, temperature, etc

## [0.3.4] - 2023-02-06
### Added
- add millivolts and milliwatts to the device structs for precise readings
