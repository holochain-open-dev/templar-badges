const path = require("path");
const tape = require("tape");

const {
  Orchestrator,
  Config,
  tapeExecutor,
  localOnly,
  combine
} = require("@holochain/tryorama");

const {
  createBadgeClass,
  claimUserDeservesBadge,
  getEntry,
  testBadgeClass,
  receiveOwnBadge
} = require("./utils");

process.on("unhandledRejection", error => {
  // Will print "unhandledRejection err is not defined"
  console.error("got unhandledRejection:", error);
});

const dnaPath = path.join(__dirname, "../dist/dna.dna.json");

const orchestrator = new Orchestrator({
  waiter: {
    softTimeout: 30000,
    hardTimeout: 40000
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

    const addr = await createBadgeClass()(alice);

    await s.consistency();

    let result = await getEntry(addr.Ok)(bob);

    const badgeClass = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badgeClass, {
      ...testBadgeClass,
      creator_address: aliceAddress
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

    const claimAddr = await claimUserDeservesBadge(bobAddress, addr.Ok)(alice);

    await s.consistency();

    result = await getEntry(claimAddr.Ok)(alice);

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

    result = await getEntry(claimAddr.Ok)(alice);

    badge = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badge, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuers: [aliceAddress],
      evidences: []
    });

    await s.consistency();

    const badgeAddr = await receiveOwnBadge(addr.Ok)(bob);
    t.ok(badgeAddr.Ok);
    await s.consistency();

    result = await getEntry(claimAddr.Ok)(bob);

    badge = JSON.parse(result.Ok.App[1]);
    t.deepEqual(badge, {
      recipient: bobAddress,
      badge_class: addr.Ok,
      issuers: [aliceAddress],
      evidences: []
    });

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
    ]);
  }
);

orchestrator.registerScenario(
  "validation via social triangulation",
  async (s, t) => {
    const { alice, bob, carol, dave, eve } = await s.players(
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
    const addr = await createBadgeClass()(alice);
    await s.consistency();

    // Dave cannot claim that Bob deserves that badge
    let error = await claimUserDeservesBadge(bobAddress, addr.Ok)(dave);
    t.notOk(error.Ok);
    await s.consistency();

    // Alice claims that Bob should get the badge
    let result = await claimUserDeservesBadge(bobAddress, addr.Ok)(alice);
    t.ok(result.Ok);
    await s.consistency();

    result = await getEntry(result.Ok)(bob);
    const badge = JSON.parse(result.Ok.App[1]);
    t.equal(badge.issuers.length, 1);

    result = await receiveOwnBadge(addr.Ok)(bob);
    t.ok(result.Ok);
    await s.consistency();

    // Alice claims that Carol should get the badge
    result = await claimUserDeservesBadge(carolAddress, addr.Ok)(alice);
    t.ok(result.Ok);
    await s.consistency();

    result = await receiveOwnBadge(addr.Ok)(carol);
    t.ok(result.Ok);
    await s.consistency();

    // Bob now claims that Dave should get the badge
    result = await claimUserDeservesBadge(daveAddress, addr.Ok)(bob);
    t.ok(result.Ok);
    await s.consistency();

    // Carol now claims that Dave should get the badge
    result = await claimUserDeservesBadge(daveAddress, addr.Ok)(carol);
    t.ok(result.Ok);
    await s.consistency();

    result = await receiveOwnBadge(addr.Ok)(dave);
    t.ok(result.Ok);
    await s.consistency();

    // Carol now claims that Eve should get the badge
    result = await claimUserDeservesBadge(eveAddress, addr.Ok)(carol);
    t.ok(result.Ok);
    await s.consistency();

    // Dave now claims that Eve should get the badge
    result = await claimUserDeservesBadge(eveAddress, addr.Ok)(dave);
    t.ok(result.Ok);
    await s.consistency();

    result = await receiveOwnBadge(addr.Ok)(eve);
    t.ok(result.Ok);
    await s.consistency();
  }
);

orchestrator.run();
