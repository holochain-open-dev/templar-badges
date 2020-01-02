import { GraphQlSchemaModule } from '@uprtcl/common';
import {
  ElementsModule,
  MicroModule,
  i18nextModule
} from '@uprtcl/micro-orchestrator';

import { BadgesForAgent } from './elements/badges-for-agent';

import en from '../i18n/en.json';
import { badgesTypeDefs } from './graphql';

export class BadgesModule extends MicroModule {
  static id = Symbol('badges-module');

  constructor(badgesConfig) {
    super();
  }

  submodules = [
    new GraphQlSchemaModule(badgesTypeDefs, {}),
    new i18nextModule('badges', { en: en }),
    new ElementsModule({
      'badges-for-agent': BadgesForAgent
    })
  ];
}
