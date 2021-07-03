import "./imports/Escrow.sol";

// File: contracts/EscrowMatic.sol


pragma solidity ^0.7.6;





/**
 * @title Escrow
 * @dev Escrow contract that works with RNDR token
 * This contract holds tokens while render jobs are being completed
 * and information on token allottment per user
 */
contract Escrow is OwnableUpgradeable {
  using SafeERC20Upgradeable for IERC20Upgradeable;
  using SafeMathUpgradeable for uint256;

  // This is a mapping of user IDs to the number of tokens held in escrow
  mapping(string => uint256) private userBalances;
  // This is the address of the render token contract
  address public renderTokenAddress;
  // This is the address with authority to call the disburseFunds function
  address public disbursalAddress;

  // Emit new disbursal address when disbursalAddress has been changed
  event DisbursalAddressUpdate(address disbursalAddress);
  // Emit the userId along with the new balance of the user escrow
  // Internal systems will watch this event to determine balances available
  event UserBalanceUpdate(string _userId, uint256 _balance);
  // Emit new contract address when renderTokenAddress has been changed
  event RenderTokenAddressUpdate(address renderTokenAddress);

  /**
   * @dev Modifier to check if the message sender can call the disburseFunds function
   */
  modifier canDisburse() {
    require(_msgSender() == disbursalAddress, "message sender not authorized to disburse funds");
    _;
  }

  /**
   * @dev Calling initialize in logic implementation contract
   */
  constructor() {
    initialize(address(1), address(1));
  }

  /**
   * @dev Initailization
   * @param _owner because this contract uses proxies, owner must be passed in as a param
   * @param _renderTokenAddress see renderTokenAddress
   */
  function initialize (address _owner, address _renderTokenAddress) public initializer {
    require(_owner != address(0), "_owner must not be null");
    require(_renderTokenAddress != address(0), "_renderTokenAddress must not be null");
    __Ownable_init();
    OwnableUpgradeable.transferOwnership(_owner);
    disbursalAddress = _owner;
    renderTokenAddress = _renderTokenAddress;
  }

  /**
   * @dev Change the address authorized to distribute tokens for completed jobs
   *
   * Because there are no on-chain details to indicate who performed a render, an outside
   * system must call the disburseFunds function with the information needed to properly
   * distribute tokens. This function updates the address with the authority to perform distributions
   * @param _newDisbursalAddress see disbursalAddress
   */
  function changeDisbursalAddress(address _newDisbursalAddress) external onlyOwner {
    disbursalAddress = _newDisbursalAddress;

    emit DisbursalAddressUpdate(disbursalAddress);
  }

  /**
   * @dev Change the address allowances will be sent to after job completion
   *
   * Ideally, this will not be used, but is included as a failsafe.
   * RNDR is still in its infancy, and changes may need to be made to this
   * contract and / or the renderToken contract. Including methods to update the
   * addresses allows the contracts to update independently.
   * If the RNDR token contract is ever migrated to another address for
   * either added security or functionality, this will need to be called.
   * @param _newRenderTokenAddress see renderTokenAddress
   */
  function changeRenderTokenAddress(address _newRenderTokenAddress) external onlyOwner {
    require(_newRenderTokenAddress != address(0), "_newRenderTokenAddress must not be null");
    renderTokenAddress = _newRenderTokenAddress;

    emit RenderTokenAddressUpdate(renderTokenAddress);
  }

  /**
   * @dev Send allowances to node(s) that performed a job
   *
   * This can only be called by the disbursalAddress, an accound owned
   * by OTOY, and it provides the number of tokens to send to each node
   * @param _userId the ID of the user used in the userBalances mapping
   * @param _recipients the address(es) of the nodes that performed rendering
   * @param _amounts the amount(s) to send to each address. These must be in the same
   * order as the recipient addresses
   */
  function disburseFunds(string calldata _userId, address[] calldata _recipients, uint256[] calldata _amounts) external canDisburse {
    require(userBalances[_userId] > 0, "_userId has no available balance");
    require(_recipients.length == _amounts.length, "_recipients and _amounts must be the same length");

    for(uint256 i = 0; i < _recipients.length; i++) {
      userBalances[_userId] = userBalances[_userId].sub(_amounts[i]);
      IERC20Upgradeable(renderTokenAddress).safeTransfer(_recipients[i], _amounts[i]);
    }

    emit UserBalanceUpdate(_userId, userBalances[_userId]);
  }

  /**
   * @dev Add RNDR tokens to a user escrow balance
   *
   * This can only be called by a function on the RNDR token contract
   * @param _userId the ID of the uesr used in the userBalances mapping
   * @param _tokens the number of tokens sent by the artist
   */
  function fundUser(string calldata _userId, uint256 _tokens) external {
    // This function can only be called by the address stored in the renderTokenAddress variable
    require(_msgSender() == renderTokenAddress, "message sender not authorized");
    userBalances[_userId] = userBalances[_userId].add(_tokens);

    emit UserBalanceUpdate(_userId, userBalances[_userId]);
  }

  /**
   * @dev See the tokens available in user escrow
   *
   * @param _userId the ID used to lookup the user escrow balance
   */
  function userBalance(string calldata _userId) external view returns(uint256) {
    return userBalances[_userId];
  }

}