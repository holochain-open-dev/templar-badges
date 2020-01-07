import { interfaces } from 'inversify';
import { GraphQlSchemaModule } from '@uprtcl/common';
import {
  ElementsModule,
  MicroModule,
  i18nextModule
} from '@uprtcl/micro-orchestrator';
import {
  HolochainConnectionModule,
  createHolochainProvider
} from '@uprtcl/connections';

import { BadgesForAgent } from './elements/badges-for-agent';

import en from '../i18n/en.json';
import { badgesTypeDefs } from './graphql/schema';
import { BadgeTypes } from './types';
import { resolvers } from './graphql/resolvers';

export class BadgesModule extends MicroModule {
  static id = Symbol('badges-module');

  dependencies = [HolochainConnectionModule.id];

  static types = BadgeTypes;

  constructor(protected instance: string) {
    super();
  }

  async onLoad(container: interfaces.Container) {
    const badgeProvider = createHolochainProvider(this.instance, 'badges');

    container.bind(BadgeTypes.BadgeProvider).to(badgeProvider);
  }

  submodules = [
    new GraphQlSchemaModule(badgesTypeDefs, resolvers),
    new i18nextModule('badges', { en: en }),
    new ElementsModule({
      'badges-for-agent': BadgesForAgent
    })
  ];
}
