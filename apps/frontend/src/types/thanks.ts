export type ThanksCategory = 'host' | 'mentor' | 'feedback';

export interface ThanksMember {
  name: string;
  category: ThanksCategory;
}
