import { HolochainProvider } from '@uprtcl/connections';

import { BadgeTypes } from '../types';

export const resolvers = {
  BadgeClass: {
    id(parent) {
      return parent;
    },

    async allBadges(parent, _, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_badges_for_class', {
        badge_class: parent
      });
    },
    async badge(parent, { agentAddress }, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_badge', {
        badge_class: parent,
        recipient: agentAddress
      });
    }
  },
  Query: {
    async allBadgeClasses(_, __, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_all_badge_classes', {});
    },
    async me(_, __, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_my_address', {});
    }
  },
  Mutation: {
    async createBadgeClass(_, { input }, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('create_badge_class', input);
    },
    async claimAgentDeservesBadge(
      _,
      { recipientAgent, badgeClassId, evidences },
      { container }
    ) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('claim_user_deser', {
        recipient: recipientAgent,
        badge_class: badgeClassId,
        evidences: evidences
      });
    },
    async receiveOwnBadge(_, { badgeClassId }, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('receive_own_badge', {
        badge_class: badgeClassId
      });
    }
  },
  Agent: {
    id(parent) {
      return parent;
    },
    async issuedBadges(parent, _, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_badges_from_issuer', {
        agent_address: parent
      });
    },
    async receivedBadges(parent, _, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_badges_to_recipient', {
        agent_address: parent
      });
    },
    async createdBadgeClasses(parent, _, { container }) {
      const badgeProvider: HolochainProvider = container.get(
        BadgeTypes.BadgeProvider
      );

      return badgeProvider.call('get_created_badges', {
        agent_address: parent
      });
    }
  }
};
