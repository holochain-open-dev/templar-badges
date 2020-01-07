import { gql } from 'apollo-boost';

export const badgesTypeDefs = gql`
  type Agent {
    name: String!
    issuedBadges: [Badge!]!
    receivedBadges: [Badge!]!
    createdBadgeClasses: [BadgeClass!]!
  }

  type BadgeClass {
    id: ID!

    name: String!
    description: String!
    creator: Agent!
    image: string!
    validators: Int!

    badges: [Badge!]!
  }

  type Badge {
    id: ID!

    recipient: [Agent!]!
    evidences: [Entity!]!
    issuers: [Agent!]!

    class: BadgeClass!
  }

  extend type Query {
    allBadgeClasses: [BadgeClass!]!
  }
`;
