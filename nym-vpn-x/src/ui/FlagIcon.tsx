import clsx from 'clsx';
import MsIcon from './MsIcon';

// ISO 3166-1 alpha-2 codes (two-letter country codes)
const ISO3166_1Alpha2_Codes = {
  af: 'AFGHANISTAN',
  ax: 'ÅLAND ISLANDS',
  al: 'ALBANIA',
  dz: 'ALGERIA',
  as: 'AMERICAN SAMOA',
  ad: 'ANDORRA',
  ao: 'ANGOLA',
  ai: 'ANGUILLA',
  aq: 'ANTARCTICA',
  ag: 'ANTIGUA AND BARBUDA',
  ar: 'ARGENTINA',
  am: 'ARMENIA',
  aw: 'ARUBA',
  au: 'AUSTRALIA',
  at: 'AUSTRIA',
  az: 'AZERBAIJAN',
  bs: 'BAHAMAS',
  bh: 'BAHRAIN',
  bd: 'BANGLADESH',
  bb: 'BARBADOS',
  by: 'BELARUS',
  be: 'BELGIUM',
  bz: 'BELIZE',
  bj: 'BENIN',
  bm: 'BERMUDA',
  bt: 'BHUTAN',
  bo: 'BOLIVIA, PLURINATIONAL STATE OF',
  bq: 'BONAIRE, SINT EUSTATIUS AND SABA',
  ba: 'BOSNIA AND HERZEGOVINA',
  bw: 'BOTSWANA',
  bv: 'BOUVET ISLAND',
  br: 'BRAZIL',
  io: 'BRITISH INDIAN OCEAN TERRITORY',
  bn: 'BRUNEI DARUSSALAM',
  bg: 'BULGARIA',
  bf: 'BURKINA FASO',
  bi: 'BURUNDI',
  kh: 'CAMBODIA',
  cm: 'CAMEROON',
  ca: 'CANADA',
  cv: 'CAPE VERDE',
  ky: 'CAYMAN ISLANDS',
  cf: 'CENTRAL AFRICAN REPUBLIC',
  td: 'CHAD',
  cl: 'CHILE',
  cn: 'CHINA',
  cx: 'CHRISTMAS ISLAND',
  cc: 'COCOS (KEELING) ISLANDS',
  co: 'COLOMBIA',
  km: 'COMOROS',
  cg: 'CONGO',
  cd: 'CONGO, THE DEMOCRATIC REPUBLIC OF THE',
  ck: 'COOK ISLANDS',
  cr: 'COSTA RICA',
  ci: "CÔTE D'IVOIRE",
  hr: 'CROATIA',
  cu: 'CUBA',
  cw: 'CURAÇAO',
  cy: 'CYPRUS',
  cz: 'CZECH REPUBLIC',
  dk: 'DENMARK',
  dj: 'DJIBOUTI',
  dm: 'DOMINICA',
  do: 'DOMINICAN REPUBLIC',
  ec: 'ECUADOR',
  eg: 'EGYPT',
  sv: 'EL SALVADOR',
  gq: 'EQUATORIAL GUINEA',
  er: 'ERITREA',
  ee: 'ESTONIA',
  et: 'ETHIOPIA',
  fk: 'FALKLAND ISLANDS (MALVINAS)',
  fo: 'FAROE ISLANDS',
  fj: 'FIJI',
  fi: 'FINLAND',
  fr: 'FRANCE',
  gf: 'FRENCH GUIANA',
  pf: 'FRENCH POLYNESIA',
  tf: 'FRENCH SOUTHERN TERRITORIES',
  ga: 'GABON',
  gm: 'GAMBIA',
  ge: 'GEORGIA',
  de: 'GERMANY',
  gh: 'GHANA',
  gi: 'GIBRALTAR',
  gr: 'GREECE',
  gl: 'GREENLAND',
  gd: 'GRENADA',
  gp: 'GUADELOUPE',
  gu: 'GUAM',
  gt: 'GUATEMALA',
  gg: 'GUERNSEY',
  gn: 'GUINEA',
  gw: 'GUINEA-BISSAU',
  gy: 'GUYANA',
  ht: 'HAITI',
  hm: 'HEARD ISLAND AND MCDONALD ISLANDS',
  va: 'HOLY SEE (VATICAN CITY STATE)',
  hn: 'HONDURAS',
  hk: 'HONG KONG',
  hu: 'HUNGARY',
  is: 'ICELAND',
  in: 'INDIA',
  id: 'INDONESIA',
  ir: 'IRAN, ISLAMIC REPUBLIC OF',
  iq: 'IRAQ',
  ie: 'IRELAND',
  im: 'ISLE OF MAN',
  il: 'ISRAEL',
  it: 'ITALY',
  jm: 'JAMAICA',
  jp: 'JAPAN',
  je: 'JERSEY',
  jo: 'JORDAN',
  kz: 'KAZAKHSTAN',
  ke: 'KENYA',
  ki: 'KIRIBATI',
  kp: "KOREA, DEMOCRATIC PEOPLE'S REPUBLIC OF",
  kr: 'KOREA, REPUBLIC OF',
  kw: 'KUWAIT',
  kg: 'KYRGYZSTAN',
  la: "LAO PEOPLE'S DEMOCRATIC REPUBLIC",
  lv: 'LATVIA',
  lb: 'LEBANON',
  ls: 'LESOTHO',
  lr: 'LIBERIA',
  ly: 'LIBYA',
  li: 'LIECHTENSTEIN',
  lt: 'LITHUANIA',
  lu: 'LUXEMBOURG',
  mo: 'MACAO',
  mk: 'MACEDONIA, THE FORMER YUGOSLAV REPUBLIC OF',
  mg: 'MADAGASCAR',
  mw: 'MALAWI',
  my: 'MALAYSIA',
  mv: 'MALDIVES',
  ml: 'MALI',
  mt: 'MALTA',
  mh: 'MARSHALL ISLANDS',
  mq: 'MARTINIQUE',
  mr: 'MAURITANIA',
  mu: 'MAURITIUS',
  yt: 'MAYOTTE',
  mx: 'MEXICO',
  fm: 'MICRONESIA, FEDERATED STATES OF',
  md: 'MOLDOVA, REPUBLIC OF',
  mc: 'MONACO',
  mn: 'MONGOLIA',
  me: 'MONTENEGRO',
  ms: 'MONTSERRAT',
  ma: 'MOROCCO',
  mz: 'MOZAMBIQUE',
  mm: 'MYANMAR',
  na: 'NAMIBIA',
  nr: 'NAURU',
  np: 'NEPAL',
  nl: 'NETHERLANDS',
  nc: 'NEW CALEDONIA',
  nz: 'NEW ZEALAND',
  ni: 'NICARAGUA',
  ne: 'NIGER',
  ng: 'NIGERIA',
  nu: 'NIUE',
  nf: 'NORFOLK ISLAND',
  mp: 'NORTHERN MARIANA ISLANDS',
  no: 'NORWAY',
  om: 'OMAN',
  pk: 'PAKISTAN',
  pw: 'PALAU',
  ps: 'PALESTINE, STATE OF',
  pa: 'PANAMA',
  pg: 'PAPUA NEW GUINEA',
  py: 'PARAGUAY',
  pe: 'PERU',
  ph: 'PHILIPPINES',
  pn: 'PITCAIRN',
  pl: 'POLAND',
  pt: 'PORTUGAL',
  pr: 'PUERTO RICO',
  qa: 'QATAR',
  re: 'RÉUNION',
  ro: 'ROMANIA',
  ru: 'RUSSIAN FEDERATION',
  rw: 'RWANDA',
  bl: 'SAINT BARTHÉLEMY',
  sh: 'SAINT HELENA, ASCENSION AND TRISTAN DA CUNHA',
  kn: 'SAINT KITTS AND NEVIS',
  lc: 'SAINT LUCIA',
  mf: 'SAINT MARTIN (FRENCH PART)',
  pm: 'SAINT PIERRE AND MIQUELON',
  vc: 'SAINT VINCENT AND THE GRENADINES',
  ws: 'SAMOA',
  sm: 'SAN MARINO',
  st: 'SAO TOME AND PRINCIPE',
  sa: 'SAUDI ARABIA',
  sn: 'SENEGAL',
  rs: 'SERBIA',
  sc: 'SEYCHELLES',
  sl: 'SIERRA LEONE',
  sg: 'SINGAPORE',
  sx: 'SINT MAARTEN (DUTCH PART)',
  sk: 'SLOVAKIA',
  si: 'SLOVENIA',
  sb: 'SOLOMON ISLANDS',
  so: 'SOMALIA',
  za: 'SOUTH AFRICA',
  gs: 'SOUTH GEORGIA AND THE SOUTH SANDWICH ISLANDS',
  ss: 'SOUTH SUDAN',
  es: 'SPAIN',
  lk: 'SRI LANKA',
  sd: 'SUDAN',
  sr: 'SURINAME',
  sj: 'SVALBARD AND JAN MAYEN',
  sz: 'SWAZILAND',
  se: 'SWEDEN',
  ch: 'SWITZERLAND',
  sy: 'SYRIAN ARAB REPUBLIC',
  tw: 'TAIWAN, PROVINCE OF CHINA',
  tj: 'TAJIKISTAN',
  tz: 'TANZANIA, UNITED REPUBLIC OF',
  th: 'THAILAND',
  tl: 'TIMOR-LESTE',
  tg: 'TOGO',
  tk: 'TOKELAU',
  to: 'TONGA',
  tt: 'TRINIDAD AND TOBAGO',
  tn: 'TUNISIA',
  tr: 'TURKEY',
  tm: 'TURKMENISTAN',
  tc: 'TURKS AND CAICOS ISLANDS',
  tv: 'TUVALU',
  ug: 'UGANDA',
  ua: 'UKRAINE',
  ae: 'UNITED ARAB EMIRATES',
  gb: 'UNITED KINGDOM',
  us: 'UNITED STATES',
  um: 'UNITED STATES MINOR OUTLYING ISLANDS',
  uy: 'URUGUAY',
  uz: 'UZBEKISTAN',
  vu: 'VANUATU',
  ve: 'VENEZUELA, BOLIVARIAN REPUBLIC OF',
  vn: 'VIET NAM',
  vg: 'VIRGIN ISLANDS, BRITISH',
  vi: 'VIRGIN ISLANDS, U.S.',
  wf: 'WALLIS AND FUTUNA',
  eh: 'WESTERN SAHARA',
  ye: 'YEMEN',
  zm: 'ZAMBIA',
  zw: 'ZIMBABWE',
} as const;

// two-letter country code (ISO 3166-1 alpha-2)
export type countryCode = keyof typeof ISO3166_1Alpha2_Codes;

function isAlpha2Code(code: string): code is countryCode {
  return code in ISO3166_1Alpha2_Codes;
}

export type FlagIconProps = {
  className?: string;
  // two-letter country code (ISO 3166-1 alpha-2)
  code: countryCode;
  alt: string;
};

function FlagIcon({ code, alt, className }: FlagIconProps) {
  if (!isAlpha2Code(code)) {
    return (
      <MsIcon
        icon="broken_image"
        className={clsx(['h-7 w-7 min-w-7', className && className])}
      />
    );
  }

  return (
    <div className="w-7 min-w-7 flex justify-center items-center">
      <img
        src={`./flags/${code}.svg`}
        className={clsx([
          'h-7 scale-90 pointer-events-none fill-current',
          className && className,
        ])}
        alt={alt}
      />
    </div>
  );
}

export default FlagIcon;
