# Change log

## [1.3.1] 2018-10-03

### Added
    - Add the command to return a relative measurment of reception power
    - Add the ranging correction methods and data

## [1.3] 2018-07-31

### Added
    - Add LNA configuration
    - Add AGC commands configuration: enable/disable and set gain step

### Fixed
    - Remove the piece of code that was reversing endianess in UART init method

## [1.2] 2018-03-30

### Added
    - Add method to disable AutoTx operations (fix #2)
    - Add method to set SPI speed in HAL (#1)

### Changed
    - Update the BLE connection state tokens to align with datasheet v2.0 (#8)

### Fixed
    - Fix description of GetPacketStatus command (#7)
    - Fix bug (#5) with compile failed with GCC due to wrong declaration of RadioCommands_t in radio.h file

## [1.1] 2017-09-26

### Added
    - Add methods to set access address for BLE advertisement operations

### Fixed
    - Fix the payload size return by GetRxBufferStatus for BLE packet

## [1.0.2] 2017-08-04

### Removed
    - Remove unused variable ContinuousMode and SingleMode

### Fixed
    - Fix wrong register read for firmware version

## [1.0.1] 2017-06-23

### Fixed
    - Wrong value for SingleMode

## [1.0] 2017-05-22

### Added
    - Code comments
    - The following commands have been addded:
        - SetSyncWordErrorTolerance to configure the number of wrong bits allowed in sync word detection
        - SetAutoFs to set the chip in Frequency Synthesis mode after packet transmission
        - SetRangingRole to set the chip in the corresponding role in ranging exchange
        - GetFrequencyError to get the frequency deviation based on last received LoRa packet

### Changed
    - The modulation names have been updated to fit with Datasheet
    - The offset of SendPayload command is directly accessible through the API (default to 0)
    - The GetPacketType can be recovered from the chip or from the last saved value in the driver, allowing less SPI transfert
    - The GetRangignResult returns distance in meter respecting the datasheet formula

### Removed
    - Remove all commands related to patch ram
    - Remove all workarounds for previous chip version

### Fixed
    - The packet status command do not return wrong status for LoRa and RSSI average on other packet type
    - Removed some FLRC datarates
    - Modify the IRQ management to trigger the correct callback
    - The SetRxDutyCycle is now defined so that Sleep and Rx share a common period base
    - Correct the SPI call for GetStatus to ensure the correct value is returned
    - Change the DIO states to help reduce the chip current consumption


## [0.9.0]
First release of SX1280 driver in C++ on top of Mbed
