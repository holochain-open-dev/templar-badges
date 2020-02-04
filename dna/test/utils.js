let testBadgeClass = {
  name: "Test badge",
  description: "Test description",
  image: "Test image",
  validators: 2
};

function createBadgeClass(badgeClass = testBadgeClass) {
  return caller =>
    caller.call("badges_instance", "badges", "create_badge_class", badgeClass);
}

function claimAgentDeservesBadge(recipient, badgeClass) {
  return caller =>
    caller.call("badges_instance", "badges", "claim_agent_deserves_badge", {
      recipient: recipient,
      badge_class: badgeClass,
      evidences: []
    });
}

function getEntry(address) {
  return caller =>
    caller.call("badges_instance", "badges", "get_entry", {
      address
    });
}

function getEntryHistory(address) {
  return caller => {
    const entry = caller.call(
      "badges_instance",
      "badges",
      "get_entry_history",
      {
        address
      }
    );

    return JSON.parse(entry.Ok.App[1]);
  };
}

function getEntries(addresses) {
  return caller => {
    const promises = addresses.map(async address => getEntry(address)(caller));

    return Promise.all(promises);
  };
}

module.exports = {
  createBadgeClass,
  claimAgentDeservesBadge,
  getEntry,
  getEntryHistory,
  testBadgeClass,
  getEntries
};
