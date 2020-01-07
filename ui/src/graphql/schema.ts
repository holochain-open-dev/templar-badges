import { gql } from 'apollo-boost';

export const badgesTypeDefs = gql`
  type Agent {
    id: ID!
    
    issuedBadges: [Badge!]!
    receivedBadges: [Badge!]!
    createdBadgeClasses: [BadgeClass!]!
  }

  type BadgeClass implements Entity {
    id: ID!

    name: String!
    description: String!
    creator: Agent!
    image: string!
    validators: Int!

    allBadges: [Badge!]!
    badge(agentAddress: ID!): Badge!
  }

  type Badge implements Entity {
    id: ID!

    recipient: [Agent!]!
    evidences: [Entity!]!
    issuers: [Agent!]!

    class: BadgeClass!
  }

  extend type Query {
    allBadgeClasses: [BadgeClass!]!
    me: Agent!
    agent(agentId: ID!): Agent!
  }

  input BadgeClassInput {
    name: String!
    description: String!
    image: String!
    validators: Int!
  }

  extend type Mutation {
    createBadgeClass(input: BadgeClassInput!): BadgeClass!
    claimAgentDeservesBadge(
      recipientAgent: ID!
      badgeClassId: ID!
      evidences: [ID!]!
    ): Badge!
    receiveOwnBadge(badgeClassId: ID!): Badge!
  }
`;
