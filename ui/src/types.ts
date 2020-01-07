export const BadgeTypes = {
  BadgeProvider: Symbol('badge-provider')
};

export interface Badge {
  id: string;
  recipient: string;
  issuers: string[];

  class: BadgeClass;
}

export interface BadgeClass {
  id: string;
  name: string;
  description: string;
  image: string;
}
