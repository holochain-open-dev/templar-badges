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

function receiveOwnBadge(badgeClass) {
  return caller =>
    caller.call("badges_instance", "badges", "receive_own_badge", {
      badge_class: badgeClass
    });
}

function isOwnBadgeValid(badgeClass) {
  return caller =>
    caller.call("badges_instance", "badges", "is_own_badge_valid", {
      badge_class: badgeClass
    });
}

function getEntry(address) {
  return caller =>
    caller.call("badges_instance", "badges", "get_entry", {
      address
    });
}

function getEntryHistory(address) {
  return caller =>
    caller.call("badges_instance", "badges", "get_entry_history", {
      address
    });
}

module.exports = {
  createBadgeClass,
  claimAgentDeservesBadge,
  getEntry,
  receiveOwnBadge,
  getEntryHistory,
  isOwnBadgeValid,
  testBadgeClass
};
