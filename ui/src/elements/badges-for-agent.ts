import { LitElement, property, html } from 'lit-element';
import { ApolloClient, gql } from 'apollo-boost';

import { sharedStyles } from '@uprtcl/lenses';
import { ApolloClientModule } from '@uprtcl/common';
import { moduleConnect } from '@uprtcl/micro-orchestrator';

import '@material/mwc-top-app-bar';
import { GET_AGENT_BADGES } from 'src/graphql/queries';
import { Badge } from '../types';

export class BadgesForAgent extends moduleConnect(LitElement) {
  @property({ type: String })
  agentId!: string;

  @property({ type: Object })
  badges!: Array<Badge>;

  async firstUpdated() {
    const client: ApolloClient<any> = this.request(
      ApolloClientModule.types.Client
    );
    const result = await client.query({
      query: GET_AGENT_BADGES,
      variables: {
        agentId: this.agentId
      }
    });

    this.badges = result.data.agent.receivedBadges;
  }

  render() {
    if (!this.badges)
      return html`
        <span>Loading...</span>
      `;

    return html`
      <div class="column">
        ${this.badges.map(
          badge => html`
            <div>
              ${badge.class.name}
            </div>
          `
        )}
      </div>
    `;
  }

  static get styles() {
    return sharedStyles;
  }
}
