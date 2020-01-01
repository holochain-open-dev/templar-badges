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

function claimUserDeservesBadge(recipient, badgeClass) {
  return caller =>
    caller.call("badges_instance", "badges", "claim_user_deserves_badge", {
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

function getEntry(address) {
  return caller =>
    caller.call("badges_instance", "badges", "get_entry", {
      address
    });
}

module.exports = {
  createBadgeClass,
  claimUserDeservesBadge,
  getEntry,
  receiveOwnBadge,
  testBadgeClass
};
