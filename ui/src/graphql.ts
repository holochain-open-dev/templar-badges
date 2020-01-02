import { gql } from 'apollo-boost';

export const badgesTypeDefs = gql`
  type Agent {
    name: String!
    badges: [Badge!]!
  }
  
  type BadgeClass {
    name: String!
    description: String!
    creator: Agent!
    image: string!
    validators: Int!

    badges: [Badge!]!
  }

  type Badge {
    recipient: [Agent!]!
    evidences: [Entity!]!
    issuers: [Agent!]!

    class: BadgeClass!
  }

  extend union EntityType = Badge | BadgeClass
`;