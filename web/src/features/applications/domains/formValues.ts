export interface DomainFormValues {
  host: string;
  port: string;
  path: string;
  https: boolean;
  service: string;
  protect: boolean;
  authUser: string;
  authPassword: string;
}

export const EMPTY_DOMAIN_FORM: DomainFormValues = {
  host: "",
  port: "80",
  path: "",
  https: true,
  service: "",
  protect: false,
  authUser: "",
  authPassword: "",
};
