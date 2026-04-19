## [v0.3] - 2026-04-19

### Added
- Explicit `Domain` enum with values:
  - Hash = 0x01
  - Xof = 0x02
  - TreeLeaf = 0x03
  - TreeParent = 0x04
  - MacKeyed = 0x05
  - Transcript = 0x06
  - Artifact = 0x07
  - RoundTrace = 0x08
- Published round constants (K0_0, K1_0, K2_0) in vectors.json

### Changed
- Plain hash now uses explicit domain byte 0x01
- **Breaking**: All hash outputs differ from v0.2

### Deprecated
- v0.2 vectors are archival only; do not use for validation
