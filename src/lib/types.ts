export type MatchData = {
  ingame: boolean;
  map: string;
  mode: string;
  players: Player[];
};

export type Player = {
  uuid: string;
  name: string;
  team: "blue" | "red" | "unknown";
  character?: Agent;
  title: string;
  accountLevel?: number;
  rankHistory: CompetitiveTier[];
};

export type Agent = {
  uuid: string;
  displayName: string;
  displayIcon: string;
};

export type CompetitiveTier = {
  episode: string;
  tier: number;
  tierName?: string;
  largeIcon?: string;
};
