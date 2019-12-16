const path = require("path");
const tape = require("tape");

const {
  Orchestrator,
  Config,
  tapeExecutor,
  localOnly,
  combine
} = require("@holochain/tryorama");

process.on("unhandledRejection", error => {
  // Will print "unhandledRejection err is not defined"
  console.error("got unhandledRejection:", error);
});

const dnaPath = path.join(__dirname, "../dist/dna.dna.json");

const orchestrator = new Orchestrator();

const mainConfig = Config.gen(
  {
    badges_instance: Config.dna(dnaPath, "scaffold-test")
  },
  {
    network: {
      type: "sim2h",
      sim2h_url: "ws://localhost:9000"
    }
  }
);

orchestrator.registerScenario(
  "create badge class, make a badge claim for another agent, have that agent create the badge assertion",
  async (s, t) => {
    const { alice, bob, carol } = await s.players(
      {
        alice: mainConfig,
        bob: mainConfig,
        carol: mainConfig,
      },
      true
    );

    const aliceAddress = alice.instance("badges_instance").agentAddress;
    const bobAddress = bob.instance("badges_instance").agentAddress;

    const addr = await alice.call(
      "badges_instance",
      "badges",
      "create_badge_class",
      {
        name: "Test badge",
        description: "Test description",
        image: "Test image",
        validators: 2
      }
    );

    await s.consistency();

    let result = await alice.call("badges_instance", "badges", "get_entry", {
      address: addr.Ok
    });

    const badgeClass = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badgeClass, {
      name: "Test badge",
      description: "Test description",
      image: "Test image",
      creator_address: aliceAddress,
      validators: 2
    });

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_all_badge_classes",
      {}
    );

    const allBadges = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(allBadges, [
      {
        name: "Test badge",
        description: "Test description",
        image: "Test image",
        creator_address: aliceAddress,
        validators: 2
      }
    ]);

    const claimAddr = await alice.call(
      "badges_instance",
      "badges",
      "create_badge_claim",
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );

    await s.consistency();

    result = await alice.call("badges_instance", "badges", "get_entry", {
      address: claimAddr.Ok
    });

    const badgeClaim = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badgeClaim, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuer: aliceAddress,
      evidences: []
    });

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badge_class_claims",
      {
        badge_class: addr.Ok
      }
    );

    let badgeClaims = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeClaims, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuer: aliceAddress,
        evidences: []
      }
    ]);

    const assertionAddr = await bob.call(
      "badges_instance",
      "badges",
      "create_own_badge_assertion",
      {
        badge_class: addr.Ok
      }
    );

    await s.consistency();

    console.log("iaaa", assertionAddr);

    result = await bob.call("badges_instance", "badges", "get_entry", {
      address: assertionAddr.Ok
    });

    const badgeAssertion = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badgeAssertion, {
      recipient: bobAddress,
      badge_class: addr.Ok
    });

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badge_class_assertions",
      {
        badge_class: addr.Ok
      }
    );

    let badgeAssertions = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeAssertions, [
      {
        recipient: aliceAddress,
        badge_class: addr.Ok
      },
      {
        recipient: bobAddress,
        badge_class: addr.Ok
      }
    ]);

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badge_assertions_to_recipient",
      {
        agent_address: bobAddress
      }
    );

    badgeAssertions = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeAssertions, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok
      }
    ]);

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badge_claims_from_issuer",
      {
        agent_address: aliceAddress
      }
    );

    badgeClaims = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeClaims, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuer: aliceAddress,
        evidences: []
      }
    ]);

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badge_claims_to_recipient",
      {
        agent_address: bobAddress
      }
    );

    badgeClaims = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeClaims, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuer: aliceAddress,
        evidences: []
      }
    ]);
  }
);
 
orchestrator.registerScenario(
  "validation via social triangulation",
  async (s, t) => {
    const { alice, bob, carol, dave } = await s.players(
      {
        alice: mainConfig,
        bob: mainConfig,
        carol: mainConfig,
        dave: mainConfig
      },
      true
    );

    const aliceAddress = alice.instance("badges_instance").agentAddress;
    const bobAddress = bob.instance("badges_instance").agentAddress;
    const carolAddress = carol.instance("badges_instance").agentAddress;
    const daveAddress = dave.instance("badges_instance").agentAddress;

    const addr = await alice.call(
      "badges_instance",
      "badges",
      "create_badge_class",
      {
        name: "Test badge",
        description: "Test description",
        image: "Test image",
        validators: 2
      }
    );

    await s.consistency();

    let error = await dave.call(
      "badges_instance",
      "badges",
      "create_badge_claim",
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.notOk(error.Ok);

    await s.consistency();

    error = await bob.call(
      "badges_instance",
      "badges",
      "create_own_badge_assertion",
      {
        badge_class: addr.Ok
      }
    );

    t.notOk(error.Ok);
    await s.consistency();

    let result = await alice.call(
      "badges_instance",
      "badges",
      "create_badge_claim",
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await bob.call(
      "badges_instance",
      "badges",
      "create_own_badge_assertion",
      {
        badge_class: addr.Ok
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await alice.call(
      "badges_instance",
      "badges",
      "create_badge_claim",
      {
        recipient: carolAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );

    t.ok(result.Ok);

    await s.consistency();

    result = await carol.call(
      "badges_instance",
      "badges",
      "create_own_badge_assertion",
      {
        badge_class: addr.Ok
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await bob.call("badges_instance", "badges", "create_badge_claim", {
      recipient: daveAddress,
      badge_class: addr.Ok,
      evidences: []
    });
    t.ok(result.Ok);

    await s.consistency();

    result = await carol.call(
      "badges_instance",
      "badges",
      "create_badge_claim",
      {
        recipient: daveAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await dave.call(
      "badges_instance",
      "badges",
      "get_badge_claims_to_recipient",
      {
        agent_address: daveAddress
      }
    );
    const badgeClaims = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.equal(badgeClaims.length, 2);

    await s.consistency();

    const triangulationResult = await dave.call(
      "badges_instance",
      "badges",
      "create_own_badge_assertion",
      {
        badge_class: addr.Ok
      }
    );
    await s.consistency();

    result = await dave.call("badges_instance", "badges", "get_entry", {
      address: triangulationResult.Ok
    });

    const badgeAssertion = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badgeAssertion, {
      recipient: daveAddress,
      badge_class: addr.Ok
    });
  }
); 

orchestrator.run();
