export type MatchData = {
  map: string;
  mode: string;
  players: Player[];
}

export type Player = {
  name: string;
  team: "blue" | "red" | "unknown";
  character: Agent | undefined;
  title: string;
  accountLevel: number | undefined;
  rankHistory: number[];
}

export type Agent = {
  uuid: string;
  displayName: string;
  displayIcon: string;
}