import "./imports/RenderToken.sol";

// File: contracts/RenderTokenMatic.sol

pragma solidity ^0.7.6;



interface IEscrow {
    function fundUser(string calldata _userId, uint256 _tokens) external;
}

/**
 * @title RenderToken
 * @dev ERC20 mintable token
 * The token will be minted by the crowdsale contract only
 */
contract RenderToken is ERC20Upgradeable, OwnableUpgradeable {

    // The address of the contract that manages user balances. Address is used for forwarding tokens
    // that come in to fund jobs
    address public escrowContractAddress;
    // Matic childChainManager
    address public childChainManagerProxy;

    // Emit new contract address when escrowContractAddress has been changed
    event EscrowContractAddressUpdate(address escrowContractAddress);
    // Emit information related to tokens being escrowed
    event TokensEscrowed(address indexed sender, string userId, uint256 amount);

    /**
     * @dev Calling initialize in logic implementation contract
     */
    constructor() {
        initialize(address(1), address(1), "1", "1");
    }

    /**
     * @dev Initailization
     * @param _owner Owner of the contract is able to change the Escrow address
     * @param _childChainManagerProxy Matic Child Chain Manager Proxy address
     */
    function initialize(address _owner, address _childChainManagerProxy, string memory _name, string memory _symbol) public initializer {
        require(_owner != address(0), "_owner must not be null");
        require(_childChainManagerProxy != address(0), "_childChainManagerProxy must not be null");
        __ERC20_init(_name, _symbol);
        __Ownable_init_unchained();
        OwnableUpgradeable.transferOwnership(_owner);
        childChainManagerProxy = _childChainManagerProxy;
    }

    /**
     * @dev Push tokens into an escrow
     *
     * This function is called by the artist, and it will transfer tokens
     * to a separate escrow contract to be held
     * @param _userID is the ID of the user
     * @param _amount is the number of RNDR tokens being added to escrow
     */
    function holdInEscrow(string calldata _userID, uint256 _amount) public {
        require(transfer(escrowContractAddress, _amount), "token transfer to escrow address failed");
        IEscrow(escrowContractAddress).fundUser(_userID, _amount);

        emit TokensEscrowed(_msgSender(), _userID, _amount);
    }

    /**
     * @dev Set the address of the escrow contract
     *
     * This will dictate the contract that will hold tokens in escrow and keep
     * a ledger of funds available for jobs.
     * RNDR is still in its infancy, and changes may need to be made to this
     * contract and / or the escrow contract. Including methods to update the
     * addresses allows the contracts to update independently.
     * If the escrow contract is ever migrated to another address for
     * either added security or functionality, this will need to be called.
     * @param _escrowAddress see escrowContractAddress
     */
    function setEscrowContractAddress(address _escrowAddress) public onlyOwner {
        require(_escrowAddress != address(0), "_escrowAddress must not be null");
        escrowContractAddress = _escrowAddress;

        emit EscrowContractAddressUpdate(escrowContractAddress);
    }

    ///////////////////////////////////////
    // Matic child token functions below //
    ///////////////////////////////////////

    // being proxified smart contract, most probably childChainManagerProxy contract's address
    // is not going to change ever, but still, lets keep it
    function updateChildChainManager(address newChildChainManagerProxy) external onlyOwner {
        require(newChildChainManagerProxy != address(0), "Bad ChildChainManagerProxy address");

        childChainManagerProxy = newChildChainManagerProxy;
    }

    /**
      * @notice called when token is deposited on root chain
      * @dev Should be callable only by ChildChainManager
      * Should handle deposit by minting the required amount for user
      * Make sure minting is done only by this function
      * @param user user address for whom deposit is being done
      * @param depositData abi encoded amount
      */
    function deposit(address user, bytes calldata depositData)
    external
    {
        require(_msgSender() == childChainManagerProxy, "You're not allowed to deposit");

        uint256 amount = abi.decode(depositData, (uint256));
        _mint(user, amount);
    }

    /**
      * @notice called when user wants to withdraw tokens back to root chain
      * @dev Should burn user's tokens. This transaction will be verified when exiting on root chain
      * @param amount amount of tokens to withdraw
      */
    function withdraw(uint256 amount) external {
        _burn(_msgSender(), amount);
    }
}