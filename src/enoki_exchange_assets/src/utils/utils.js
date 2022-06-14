const tokenDecimals = {
  "eICP": 12,
  "eXTC": 12,
};

export const bigIntToFloat = (value, token, decimals) => {
  if (typeof decimals === 'undefined') {
    decimals = 6;
  }
  let tokenDecimal = tokenDecimals[token];
  if (!tokenDecimal) return NaN;
  let val = value * BigInt("1" + "0".repeat(decimals)) / BigInt("1" + "0".repeat(tokenDecimal));
  let str = val.toString();
  return parseFloat(str.slice(0, -decimals) + "." + str.slice(-decimals));
}

export const bigIntToStr = (value, token, decimals, defaultVal) => {
  if (typeof defaultVal === 'undefined') {
    defaultVal = null;
  }
  let ret = bigIntToFloat(value, token, decimals);
  return isNaN(ret) ? defaultVal : ret.toString();
}

export const floatToBigInt = (value, token) => {
  let tokenDecimal = tokenDecimals[token];
  if (!tokenDecimal) return null;
  let str = value.toFixed(tokenDecimal);
  return BigInt(str.split('.')[0])
}
