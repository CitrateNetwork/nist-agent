# Maps RFC-CIT-AGENT-0001 §7.1

Feature: CapsuleRegistry — ERC-1155 mints for published capsules
  As a cooperative publisher
  I want each capsule represented as an ERC-1155 mint
  So that publication, endorsement, and revocation are recorded primitives

  Background:
    Given the CapsuleRegistry contract is deployed
    And the publisher's DID is registered

  Scenario: Publishing a capsule mints an ERC-1155 with manifest hash
    When publisher signs a publish transaction
    Then a token is minted whose id = sha256(manifest)
    And the publisher's DID is recorded as the minter

  Scenario: Cooperative endorsement is a separate signed event
    Given a capsule already minted with id 0xABCD...
    When the cooperative endorser signs an endorsement
    Then an `EndorsementAdded` event is emitted referencing 0xABCD...
    And the endorser is a different DID than the publisher

  Scenario: Revocation prevents future installs without altering history
    When publisher revokes capsule 0xABCD...
    Then future install attempts reading from CapsuleRegistry refuse
    And historical install records (in operator audit chains) remain valid

# Sprint: link citrate-chain
