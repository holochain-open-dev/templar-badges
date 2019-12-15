```rust
struct BadgeClass {
    name: String,
    description: String,
    creator_address: Address,
    image: String,
}

struct BadgeClaim {
    issuer: Address,
    recipient: Address,
    badge_class: Address,
    evidences: Vec<Address>
}

struct BadgeAssertion {
    recipient: Address,
    badge_class: Address,
}
```

## Links
assertion->claim

## Validation rules

### BadgeClass
- Create: sources.includes(creator_address)

### BadgeClaim
- Create: if recipient, okey if there are two signatures. If issuer, only if there is a badgeassertion on chain
- Update: only if issuer and only change evidences

### BadgeAssertion
- Create: only if chain has N BadgeClaims, or if is creator_address