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

const orchestrator = new Orchestrator({
  waiter: {
    softTimeout: 80000,
    hardTimeout: 160000
  }
});

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
    const { alice, bob } = await s.players(
      {
        alice: mainConfig,
        bob: mainConfig
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

    let result = await bob.call("badges_instance", "badges", "get_entry", {
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

    await s.consistency();
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
      "claim_user_deserves_badge",
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

    let badge = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badge, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuers: [aliceAddress],
      evidences: []
    });

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_for_class",
      {
        badge_class: addr.Ok
      }
    );

    let badges = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badges, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuers: [aliceAddress],
        evidences: []
      }
    ]);
    
    await s.consistency();
    
    // This works!
    result = await alice.call("badges_instance", "badges", "get_entry", {
      address: claimAddr.Ok
    });
    
    badge = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badge, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuers: [aliceAddress],
      evidences: []
    });

    await s.consistency();

    /*           const badgeAddr = await bob.call(
            "badges_instance",
            "badges",
            "receive_own_badge",
            {
              badge_class: addr.Ok
            }
            );
            
            await s.consistency();
 */

    // This fails!
    result = await bob.call("badges_instance", "badges", "get_entry", {
      address: claimAddr.Ok
    });

    badge = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badge, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuers: [aliceAddress],
      evidences: []
    });

    /* 
    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_to_recipient",
      {
        agent_address: bobAddress
      }
    );

    badgeAssertions = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeAssertions, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuers: [aliceAddress],
        evidences: []
      }
    ]);

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_from_issuer",
      {
        agent_address: aliceAddress
      }
    );

    badgeClaims = result.Ok.map(b => JSON.parse(b.Ok.App[1]));
    t.deepEqual(badgeClaims, [
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        issuers: [aliceAddress],
        evidences: []
      }
    ]); */
  }
);
/* 
orchestrator.registerScenario(
  "validation via social triangulation",
  async (s, t) => {
    const { alice, bob, carol, dave } = await s.players(
      {
        alice: mainConfig,
        bob: mainConfig,
        carol: mainConfig,
        dave: mainConfig,
        eve: mainConfig
      },
      true
    );

    const aliceAddress = alice.instance("badges_instance").agentAddress;
    const bobAddress = bob.instance("badges_instance").agentAddress;
    const carolAddress = carol.instance("badges_instance").agentAddress;
    const daveAddress = dave.instance("badges_instance").agentAddress;
    const eveAddress = eve.instance("badges_instance").agentAddress;

    // Alice creates a badge
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

    // Dave cannot claim that bob deserves that badge
    let error = await dave.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.notOk(error.Ok);

    await s.consistency();

    // Bob cannot receive that badge
    error = await bob.call("badges_instance", "badges", "receive_own_badge", {
      badge_class: addr.Ok
    });

    t.notOk(error.Ok);

    await s.consistency();

    // Alice claims that Bob should get the badge
    let result = await alice.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: bobAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await bob.call("badges_instance", "badges", "receive_own_badge", {
      badge_class: addr.Ok
    });
    t.ok(result.Ok);

    await s.consistency();

    // Alice claims that Carol should get the badge
    result = await alice.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
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
      "receive_own_badge",
      {
        badge_class: addr.Ok
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    // Bob now claims that dave should get the badge
    result = await bob.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: daveAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    // Carol now claims that dave should get the badge
    result = await carol.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: daveAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await dave.call("badges_instance", "badges", "receive_own_badge", {
      badge_class: addr.Ok
    });
    await s.consistency();

    t.ok(result.Ok);

    // Carol now claims that eve should get the badge
    result = await carol.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: eveAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    // Dave now claims that eve should get the badge
    result = await dave.call(
      "badges_instance",
      "badges",
      "claim_user_deserves_badge",
      {
        recipient: eveAddress,
        badge_class: addr.Ok,
        evidences: []
      }
    );
    t.ok(result.Ok);

    await s.consistency();

    result = await eve.call("badges_instance", "badges", "receive_own_badge", {
      badge_class: addr.Ok
    });
    await s.consistency();

    t.ok(result.Ok);
  }
); */

orchestrator.run();
