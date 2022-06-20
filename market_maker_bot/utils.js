const REGEX_PRINCIPAL_ALONE = /^((?:[a-z\d]+-[a-z\d]+)+(?:-[a-z\d]+)?)$/i;
const REGEX_PRINCIPAL = /[^a-z\d-]((?:[a-z\d]+-[a-z\d]+)+(?:-[a-z\d]+)?)[^a-z\d-]/i;
const REGEX_NAT = /(?:[,\s]|\b)([\d_]+)\s*:\s*nat(?:[,\s]|\b)/i;

export const parsePrincipal = str => {
  let result = REGEX_PRINCIPAL_ALONE.exec(str);
  if (result && result[1]) {
    return result[1];
  }
  result = REGEX_PRINCIPAL.exec(str);
  if (result && result[1]) {
    return result[1];
  }
  throw new Error(`'${str}' is not a valid principal`);
}

export const parseNat = str => {
  let result = REGEX_NAT.exec(str);
  if (result && result[1]) {
    return BigInt(result[1].replace(/_/g, ''));
  }
  throw new Error(`'${str}' is not a valid nat`);
}

const getTokenDecimals = token => 12;

export const bigIntToFloat = (value, token, decimals) => {
  if (typeof decimals === 'undefined') {
    decimals = 6;
  }
  let tokenDecimal = getTokenDecimals(token);
  if (!tokenDecimal) return NaN;
  let val = value * BigInt("1" + "0".repeat(decimals)) / BigInt("1" + "0".repeat(tokenDecimal));
  let str = val.toString();
  return parseFloat(str.slice(0, -decimals) + "." + str.slice(-decimals));
}

// use significant digits due to javascript floating point error
export const floatToBigInt = (value, token, significantDigits = 8) => {
  let tokenDecimal = getTokenDecimals(token);
  if (!tokenDecimal) return null;
  if (tokenDecimal <= significantDigits) {
    return BigInt(value.toFixed(tokenDecimal).replace(/\./g, ''));
  } else {
    return BigInt(value.toFixed(significantDigits).replace(/\./g, '') + '0'.repeat(tokenDecimal - significantDigits));
  }
}
