import gql from 'graphql-tag';

export const GET_AGENT_BADGES = gql`
  query GetAgentBadges($agentId: ID!) {
    agent(agentId: $agentId) {
      id
      receivedBadges {
        id
        recipient {
          id
        }
        issuers {
          id
        }
        class {
          id
          name
          image
          description
        }
      }
    }
  }
`;
