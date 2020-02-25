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
  return async caller => {
    const response = await caller.call(
      "badges_instance",
      "badges",
      "get_entry",
      {
        address
      }
    );
    return parseEntry(response);
  };
}

function parseEntry(response) {
  const entry = response.Ok ? response.Ok : response;
  return JSON.parse(entry.App[1]);
}

function getEntryHistory(address) {
  return async caller => {
    const response = await caller.call(
      "badges_instance",
      "badges",
      "get_entry_history",
      {
        address
      }
    );

    return response.Ok.items.map(i => parseEntry(i.entry));
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
