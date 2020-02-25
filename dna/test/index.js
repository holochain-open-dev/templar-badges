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
  claimAgentDeservesBadge,
  getEntry,
  testBadgeClass,
  getEntries,
  getEntryHistory
} = require("./utils");

process.on("unhandledRejection", error => {
  // Will print "unhandledRejection err is not defined"
  console.error("got unhandledRejection:", error);
});

const dnaPath = path.join(__dirname, "../dist/dna.dna.json");

const orchestrator = new Orchestrator({
  waiter: {
    softTimeout: 50000,
    hardTimeout: 60000
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
  "create badge class, make a badge claim for another agent",
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

    const { Ok: badgeClassAddr } = await createBadgeClass()(alice);
    t.ok(badgeClassAddr);
    await s.consistency();

    let badgeClass = await getEntry(badgeClassAddr)(bob);

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

    const allBadges = await getEntries(result.Ok)(alice);
    t.deepEqual(allBadges, [
      {
        name: "Test badge",
        description: "Test description",
        image: "Test image",
        creator_address: aliceAddress,
        validators: 2
      }
    ]);

    let { Ok: badgeAddr } = await claimAgentDeservesBadge(
      bobAddress,
      badgeClassAddr
    )(alice);
    t.ok(badgeAddr);
    await s.consistency();

    const bobsBadge = {
      recipient: bobAddress,
      badge_class: badgeClassAddr,
      issuers: [aliceAddress],
      evidences: []
    };

    badge = await getEntry(badgeAddr)(alice);
    t.deepEqual(badge, bobsBadge);

    badge = await getEntry(badgeAddr)(bob);
    t.deepEqual(badge, bobsBadge);
/* 
    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_for_class",
      {
        badge_class: badgeClassAddr
      }
    );

    let badges = await getEntries(result.Ok)(alice);
    t.deepEqual(badges, [bobsBadge]);
    await s.consistency();

    badge = await getEntry(badgeAddr)(alice);
    t.deepEqual(badge, bobsBadge);
    await s.consistency();

    result = await getEntryHistory(badgeAddr)(bob);
    t.deepEqual(result, [
      {
        recipient: bobAddress,
        badge_class: badgeClassAddr,
        issuers: [],
        evidences: []
      },
      bobsBadge
    ]);

    badge = await getEntry(badgeAddr)(bob);
    t.deepEqual(badge, bobsBadge);
    await s.consistency();

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_to_recipient",
      {
        agent_address: bobAddress
      }
    );
    // TODO: FIX TEST
    //    badges = await getEntries(result.Ok)(alice);
    //    t.deepEqual(badges, [bobsBadge]);

    result = await alice.call(
      "badges_instance",
      "badges",
      "get_badges_from_issuer",
      {
        agent_address: aliceAddress
      }
    );

    badgeClaims = await getEntries(result.Ok)(alice);
    t.deepEqual(badgeClaims, [bobsBadge]);
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
    const { Ok: badgeClassAddress } = await createBadgeClass()(alice);
    await s.consistency();

    // Dave cannot claim that Bob deserves that badge
    let error = await claimAgentDeservesBadge(
      bobAddress,
      badgeClassAddress
    )(dave);
    t.notOk(error.Ok);
    await s.consistency();

    // Alice claims that Bob should get the badge
    let result = await claimAgentDeservesBadge(
      bobAddress,
      badgeClassAddress
    )(alice);
    t.ok(result.Ok);
    await s.consistency();

    let badge = await getEntry(result.Ok)(bob);
    t.equal(badge.issuers.length, 1);

    // Alice claims that Carol should get the badge
    result = await claimAgentDeservesBadge(
      carolAddress,
      badgeClassAddress
    )(alice);
    t.ok(result.Ok);
    await s.consistency();

    // Bob now claims that Dave should get the badge
    result = await claimAgentDeservesBadge(daveAddress, badgeClassAddress)(bob);
    t.ok(result.Ok);
    await s.consistency();

    // Carol now claims that Dave should get the badge
    result = await claimAgentDeservesBadge(
      daveAddress,
      badgeClassAddress
    )(carol);
    t.ok(result.Ok);
    await s.consistency();

    badge = await getEntry(result.Ok)(bob);
    t.equal(badge.issuers.length, 2);

    // Dave now claims that Eve should get the badge
    result = await claimAgentDeservesBadge(eveAddress, badgeClassAddress)(dave);
    t.ok(result.Ok);
    await s.consistency(); */
  }
);

orchestrator.run();
