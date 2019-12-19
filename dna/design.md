```rust
struct BadgeClass {
    name: String,
    description: String,
    creator_address: Address,
    image: String,
    required_issuers: usize,
}

struct Badge {
    recipient: Address,
    badge_class: Address,
    issuers: Vec<Address>,
    evidences: Vec<Address>
}

```

## Validation rules

### BadgeClass
- Create: sources.includes(creator_address)

### Badge
- Create: anyone if  if recipient, okey if there are two signatures. If issuer, only if there is a badgeassertion on chain
- Update: only if issuer and only change evidences

### BadgeAssertion
- Create: only if chain has N BadgeClaims, or if is creator_address